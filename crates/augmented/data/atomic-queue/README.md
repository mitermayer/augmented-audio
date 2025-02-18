# atomic-queue
[![crates.io](https://img.shields.io/crates/v/atomic-queue.svg)](https://crates.io/crates/atomic-queue)
[![docs.rs](https://docs.rs/atomic-queue/badge.svg)](https://docs.rs/atomic-queue/)
- - -

Multi-producer multi-consumer bounded lock-free queue for use in Audio applications, ported from
https://github.com/max0x7ba/atomic_queue.

Quite a bit slower than `ringbuf` (~2x). This is due to this queue supporting multiple consumers and multiple producers
while `ringbuf` is single producer single consumer.

![](/criterion-screenshot.png)

## License
MIT