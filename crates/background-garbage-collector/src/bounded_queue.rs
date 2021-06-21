use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

use circular_data_structures::CircularVec;

struct BoundedQueue<T> {
    buffer: CircularVec<AtomicPtr<T>>,
    write_index: AtomicUsize,
    read_index: AtomicUsize,
    head_ptr: *mut T,
    tail_ptr: *mut T,
}

impl<T> Debug for BoundedQueue<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("BoundedQueue {\n")?;
        f.write_fmt(format_args!(
            "  write_index: {},\n",
            self.write_index.load(Ordering::Relaxed) % self.buffer.len()
        ));
        f.write_fmt(format_args!(
            "  read_index: {},\n",
            self.read_index.load(Ordering::Relaxed) % self.buffer.len()
        ));
        f.write_str("  buffer: CircularVec {\n")?;
        f.write_str("    inner: [\n");
        for i in 0..self.buffer.len() {
            f.write_str("      ")?;
            let ptr = self.buffer[i].load(Ordering::Relaxed);
            let entry_str = if i == self.read_index.load(Ordering::Relaxed) {
                "HEAD"
            } else if i == self.write_index.load(Ordering::Relaxed) {
                "TAIL"
            } else if ptr == std::ptr::null_mut() {
                "EMPTY"
            } else {
                "DATA"
            };
            f.write_str(entry_str)?;
            f.write_str(",\n")?;
        }
        f.write_str("    ]\n  }\n")?;
        f.write_str("}\n")?;
        Ok(())
    }
}

impl<T: Clone> BoundedQueue<T> {
    fn new(capacity: usize) -> Self {
        let tail_ptr = Box::into_raw(Box::new(0)) as *mut T;
        let head_ptr = Box::into_raw(Box::new(0)) as *mut T;
        let mut buffer = Vec::with_capacity(capacity + 2);
        buffer.push(AtomicPtr::new(head_ptr));
        buffer.push(AtomicPtr::new(tail_ptr));
        for _ in 0..capacity {
            buffer.push(AtomicPtr::default());
        }

        let buffer = CircularVec::with_vec(buffer);

        BoundedQueue {
            buffer,
            write_index: AtomicUsize::new(1),
            read_index: AtomicUsize::new(0),
            tail_ptr,
            head_ptr,
        }
    }

    pub fn len(&self) -> usize {
        let head_index = self.read_index.load(Ordering::Relaxed) % self.buffer.len();
        let tail_index = self.write_index.load(Ordering::Relaxed) % self.buffer.len();
        (tail_index as i32 - head_index as i32 - 1).abs() as usize
    }

    pub fn push(&self, value: *mut T) -> bool {
        let tail_index = self.write_index.load(Ordering::Acquire);
        println!("SETTING LOT {} - PTR: {:?}", tail_index, value);
        let entry = &self.buffer[tail_index];
        let entry_insert =
            entry.compare_exchange(self.tail_ptr, value, Ordering::Relaxed, Ordering::Relaxed);
        if entry_insert.is_err() {
            return false;
        }

        let change = self.buffer[tail_index + 1].compare_exchange(
            std::ptr::null_mut(),
            self.tail_ptr,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
        return if change.is_ok() {
            self.write_index.fetch_add(1, Ordering::Release);
            true
        } else {
            false
        };
    }

    pub fn read_index(&self) -> usize {
        self.read_index.load(Ordering::Relaxed)
    }

    pub fn write_index(&self) -> usize {
        self.write_index.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn mock_ptr(value: i32) -> *mut i32 {
        value as *mut i32
    }

    #[test]
    fn test_create_bounded_queue() {
        let _queue = BoundedQueue::<i32>::new(10);
    }

    #[test]
    fn test_get_empty_queue_len() {
        let queue = BoundedQueue::<i32>::new(10);
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_push_element_to_queue_increments_length() {
        let queue = BoundedQueue::<i32>::new(10);
        assert_eq!(queue.len(), 0);
        let _ = queue.push(mock_ptr(0));
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_overflow_will_not_break_things() {
        let queue = BoundedQueue::<i32>::new(3);
        assert_eq!(queue.read_index(), 0);
        assert_eq!(queue.write_index(), 1);
        assert_eq!(queue.len(), 0);

        // ENTRY 1 - HEAD, ENTRY, TAIL, EMPTY, EMPTY
        assert!(queue.push(mock_ptr(1)));
        assert_eq!(queue.read_index(), 0);
        assert_eq!(queue.write_index(), 2);
        assert_eq!(queue.len(), 1);
        println!("{:?}", queue);

        // ENTRY 2 - HEAD, ENTRY, ENTRY, TAIL, EMPTY
        assert!(queue.push(mock_ptr(2)));
        assert_eq!(queue.read_index(), 0);
        assert_eq!(queue.write_index(), 3);
        assert_eq!(queue.len(), 2);
        println!("{:?}", queue);

        // ENTRY 3 - HEAD, ENTRY, ENTRY, ENTRY, TAIL
        assert!(queue.push(mock_ptr(3)));
        assert_eq!(queue.read_index(), 0);
        assert_eq!(queue.write_index(), 4);
        assert_eq!(queue.len(), 3);
        println!("{:?}", queue);

        // ENTRY 4 - FAIL
        assert_eq!(queue.len(), 3);
        let result = queue.push(mock_ptr(666));
        println!("{:?}", queue);
        assert!(!result);
        assert_eq!(queue.len(), 3);
    }
}
