//! [`atomic_queue`] is a port of C++'s [max0x7ba/atomic_queue](https://github.com/max0x7ba/atomic_queue)
//! implementation to rust.
//!
//! It provides a bounded multi-producer, multi-consumer lock-free queue that is real-time safe.
use std::cmp::max;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicI8, AtomicUsize, Ordering};

/// State a slot in the Queue's circular buffer can be in.
enum CellState {
    Empty = 0,
    Storing = 1,
    Stored = 2,
    Loading = 3,
}

impl From<CellState> for i8 {
    fn from(value: CellState) -> Self {
        match value {
            CellState::Empty => 0,
            CellState::Storing => 1,
            CellState::Stored => 2,
            CellState::Loading => 3,
        }
    }
}

/// Atomic queue cloned from https://github.com/max0x7ba/atomic_queue
///
/// Should be:
/// * Lock-free
///
/// Any type can be pushed into the queue, but it's recommended to use some sort of smart pointer
/// that can be free-ed outside of the critical path.
///
/// Uses unsafe internally.
pub struct Queue<T> {
    head: AtomicUsize,
    tail: AtomicUsize,
    elements: Vec<MaybeUninit<T>>,
    states: Vec<AtomicI8>,
}

unsafe impl<T> Send for Queue<T> {}
unsafe impl<T> Sync for Queue<T> {}

impl<T> Queue<T> {
    /// Create a queue with a certain capacity. Writes will fail when the queue is full.
    pub fn new(capacity: usize) -> Self {
        let mut elements = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            elements.push(MaybeUninit::uninit());
        }
        let mut states = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            states.push(AtomicI8::new(CellState::Empty.into()));
        }
        let head = AtomicUsize::new(0);
        let tail = AtomicUsize::new(0);
        Queue {
            head,
            tail,
            elements,
            states,
        }
    }

    /// Push an element into the queue and return `true` on success.
    ///
    /// `false` will be returned if the queue is full. If there's contention this operation will
    /// wait until it's able to claim a slot in the queue.
    pub fn push(&self, element: T) -> bool {
        let mut head = self.head.load(Ordering::Relaxed);
        let elements_len = self.elements.len();
        loop {
            let length = head as i64 - self.tail.load(Ordering::Relaxed) as i64;
            if length >= elements_len as i64 {
                return false;
            }

            if self
                .head
                .compare_exchange(head, head + 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                self.do_push(element, head);
                return true;
            }

            head = self.head.load(Ordering::Relaxed);
        }
    }

    /// Pop an element from the queue and return `true` on success.
    ///
    /// `false` will be returned if the queue is empty. If there's contention this operation will
    /// wait until it's able to claim a slot in the queue.
    pub fn pop(&self) -> Option<T> {
        let mut tail = self.tail.load(Ordering::Relaxed);
        loop {
            let length = self.head.load(Ordering::Relaxed) as i64 - tail as i64;
            if length <= 0 {
                return None;
            }

            if self
                .tail
                .compare_exchange(tail, tail + 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }

            tail = self.tail.load(Ordering::Relaxed);
        }
        Some(self.do_pop(tail))
    }

    /// Pop an element from the queue without checking if it's empty.
    pub fn force_pop(&self) -> T {
        let tail = self.tail.fetch_add(1, Ordering::Acquire);
        self.do_pop(tail)
    }

    /// Push an element into the queue without checking if it's full.
    pub fn force_push(&self, element: T) {
        let head = self.head.fetch_add(1, Ordering::Acquire);
        self.do_push(element, head);
    }

    /// True if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the length of the queue.
    pub fn len(&self) -> usize {
        max(
            self.head.load(Ordering::Relaxed) - self.tail.load(Ordering::Relaxed),
            0,
        )
    }
}

impl<T> Queue<T> {
    fn do_pop(&self, tail: usize) -> T {
        let state = &self.states[tail % self.states.len()];
        loop {
            let expected = CellState::Stored;
            if state
                .compare_exchange(
                    expected.into(),
                    CellState::Loading.into(),
                    Ordering::Acquire,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                let self_ptr = self as *const Self as *mut Self;
                let element = unsafe {
                    std::mem::replace(
                        &mut (*self_ptr).elements[tail % self.elements.len()],
                        MaybeUninit::uninit(),
                    )
                };

                state.store(CellState::Empty.into(), Ordering::Release);

                return unsafe { element.assume_init() };
            }
        }
    }

    fn do_push(&self, element: T, head: usize) {
        self.do_push_any(element, head);
    }

    fn do_push_any(&self, element: T, head: usize) {
        let state = &self.states[head % self.states.len()];
        loop {
            let expected = CellState::Empty;
            if state
                .compare_exchange(
                    expected.into(),
                    CellState::Storing.into(),
                    Ordering::Acquire,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                unsafe {
                    let self_ptr = self as *const Self as *mut Self;
                    // There's a potential small % optimisation from removing bounds checking here &
                    // using mem::replace.
                    (*self_ptr).elements[head % self.elements.len()] = MaybeUninit::new(element);
                }
                state.store(CellState::Stored.into(), Ordering::Release);
                return;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::thread::JoinHandle;
    use std::time::Duration;

    use super::*;

    fn mock_ptr(value: i32) -> *mut i32 {
        value as *mut i32
    }

    #[test]
    fn test_create_bounded_queue() {
        let _queue = Queue::<*mut i32>::new(10);
    }

    #[test]
    fn test_get_empty_queue_len() {
        let queue = Queue::<*mut i32>::new(10);
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_push_element_to_queue_increments_length() {
        let queue = Queue::<*mut i32>::new(10);
        assert_eq!(queue.len(), 0);
        let ptr = mock_ptr(1);
        assert!(queue.push(ptr.clone()));
        assert_eq!(queue.len(), 1);
        let value = queue.pop();
        assert_eq!(value.unwrap(), ptr);
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_push_pop_push_pop() {
        let queue = Queue::<*mut i32>::new(10);
        assert_eq!(queue.len(), 0);
        {
            let ptr = mock_ptr(1);
            assert!(queue.push(ptr.clone()));
            assert_eq!(queue.len(), 1);
            let value = queue.pop();
            assert_eq!(value.unwrap(), ptr);
            assert_eq!(queue.len(), 0);
        }
        {
            let ptr = mock_ptr(2);
            assert!(queue.push(ptr.clone()));
            assert_eq!(queue.len(), 1);
            let value = queue.pop();
            assert_eq!(value.unwrap(), ptr);
            assert_eq!(queue.len(), 0);
        }
    }

    #[test]
    fn test_overflow_will_not_break_things() {
        let queue = Queue::<*mut i32>::new(3);
        assert_eq!(queue.len(), 0);

        // ENTRY 1 - HEAD, ENTRY, TAIL, EMPTY, EMPTY
        assert!(queue.push(mock_ptr(1)));
        assert_eq!(queue.len(), 1);

        // ENTRY 2 - HEAD, ENTRY, ENTRY, TAIL, EMPTY
        assert!(queue.push(mock_ptr(2)));
        assert_eq!(queue.len(), 2);

        // ENTRY 3 - HEAD, ENTRY, ENTRY, ENTRY, TAIL
        assert!(queue.push(mock_ptr(3)));
        assert_eq!(queue.len(), 3);

        // ENTRY 4 - Will fail
        assert_eq!(queue.len(), 3);
        let result = queue.push(mock_ptr(4));
        assert!(!result);
        assert_eq!(queue.len(), 3);
    }

    #[test]
    fn test_multithread_push() {
        wisual_logger::init_from_env();

        let queue = Arc::new(Queue::new(50000));

        let writer_thread_1 = spawn_writer_thread(
            10,
            queue.clone(),
            Duration::from_millis((0.0 * rand::random::<f64>()) as u64),
        );
        let writer_thread_2 = spawn_writer_thread(
            10,
            queue.clone(),
            Duration::from_millis((0.0 * rand::random::<f64>()) as u64),
        );
        let writer_thread_3 = spawn_writer_thread(
            10,
            queue.clone(),
            Duration::from_millis((0.0 * rand::random::<f64>()) as u64),
        );

        writer_thread_1.join().unwrap();
        writer_thread_2.join().unwrap();
        writer_thread_3.join().unwrap();
        assert_eq!(queue.len(), 30);
    }

    #[test]
    fn test_multithread_push_pop() {
        wisual_logger::init_from_env();

        let size = 10000;
        let num_threads = 5;

        let queue: Arc<Queue<*mut i32>> = Arc::new(Queue::new(size * num_threads / 3));
        let output_queue: Arc<Queue<*mut i32>> = Arc::new(Queue::new(size * num_threads));

        let is_running = Arc::new(Mutex::new(true));
        let reader_thread = {
            let is_running = is_running.clone();
            let queue = queue.clone();
            let output_queue = output_queue.clone();
            thread::spawn(move || {
                while *is_running.lock().unwrap() {
                    loop {
                        match queue.pop() {
                            None => break,
                            Some(value) => {
                                output_queue.push(value);
                            }
                        }
                    }
                }
                log::info!("Reader thread done reading");
            })
        };

        let threads: Vec<JoinHandle<()>> = (0..num_threads)
            .into_iter()
            .map(|_| {
                spawn_writer_thread(
                    size,
                    queue.clone(),
                    Duration::from_millis((rand::random::<f64>()) as u64),
                )
            })
            .collect();

        for thread in threads {
            thread.join().unwrap();
        }

        {
            let mut is_running = is_running.lock().unwrap();
            *is_running = false;
        }
        reader_thread.join().unwrap();

        assert_eq!(queue.len(), 0);
        assert_eq!(output_queue.len(), size * num_threads);
    }

    fn spawn_writer_thread(
        size: usize,
        queue: Arc<Queue<*mut i32>>,
        duration: Duration,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            for i in 0..size {
                loop {
                    let pushed = queue.push(mock_ptr(i as i32));
                    if pushed {
                        break;
                    }
                }
                std::thread::sleep(duration);
            }
            log::info!("Thread done writing");
        })
    }
}
