use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc,
};

use rand::Rng;

mod cb {
    use std::sync::{Condvar, Mutex};

    #[derive(PartialEq)]
    enum BarrierState {
        Receiving,
        Releasing,
    }

    struct BarrierCount {
        original: usize,
        count: usize,
        state: BarrierState,
    }

    impl BarrierCount {
        pub fn queue(&mut self) {
            self.count -= 1;
        }
        pub fn dequeue(&mut self) {
            self.count += 1;
        }
        pub fn queue_empty(&self) -> bool {
            self.count == self.original
        }
        pub fn queue_full(&self) -> bool {
            self.count == 0
        }
        pub fn release(&mut self) {
            self.state = BarrierState::Releasing;
        }
        pub fn reset(&mut self) {
            self.state = BarrierState::Receiving;
        }
        pub fn spourious(&self) -> bool {
            self.state == BarrierState::Receiving
        }
        pub fn releasing(&self) -> bool {
            self.state == BarrierState::Releasing
        }
    }

    pub struct CyclicBarrier {
        cond_var: Condvar,
        mutex: Mutex<BarrierCount>,
    }
    impl CyclicBarrier {
        pub fn new(nthreads: usize) -> CyclicBarrier {
            CyclicBarrier {
                cond_var: Condvar::new(),
                mutex: Mutex::new(BarrierCount {
                    original: nthreads,
                    count: nthreads,
                    state: BarrierState::Receiving,
                }),
            }
        }
        pub fn wait(&self) {
            let mut barrierqueue = self.mutex.lock().unwrap();
            if barrierqueue.queue_full() {
                // TODO: handle this
                panic!("Error: already waiting on maximum number of threads");
            }
            while barrierqueue.releasing() {
                // if we're releasing don't register the thread
                // just wait until we're done releasing and then allow registering
                barrierqueue = self.cond_var.wait(barrierqueue).unwrap();
            }

            // queue the thread on wait
            barrierqueue.queue();

            // if we're the last thread to queue, release all threads
            if barrierqueue.queue_full() {
                barrierqueue.release();
                self.cond_var.notify_all();
            } else {
                // otherwise wait until we're released
                // this while loop handles spourious wakeups
                // as described in the rust docs https://doc.rust-lang.org/stable/std/sync/struct.Condvar.html#method.wait
                while barrierqueue.spourious() {
                    barrierqueue = self.cond_var.wait(barrierqueue).unwrap();
                }
            }
            // dequeue the thread
            barrierqueue.dequeue();

            // if we're the last thread to dequeue, reset the barrier
            if barrierqueue.queue_empty() {
                barrierqueue.reset();
            }
        }
    }
}

fn set_speed(_speed: i32) {
    let sleep_ms = rand::thread_rng().gen_range(0..1000);
    std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
}

fn read_value() -> i32 {
    // simulate reading a value from a sensor
    let val = rand::thread_rng().gen_range(0..10);
    let sleep_ms = rand::thread_rng().gen_range(0..1000);
    std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
    val
}

fn main() {
    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
    let mut tids = vec![];
    let barrier = Arc::new(cb::CyclicBarrier::new(10));

    let reader_thread = std::thread::spawn(move || loop {
        let mut values = vec![];
        for _ in 0..10 {
            let v = match rx.recv() {
                Ok(v) => v,
                Err(_) => {
                    println!("Error: failed to receive value");
                    continue;
                }
            };
            values.push(v);
            println!("Received value: {}", v);
        }
        let sum: i32 = values.iter().sum();
        println!("Sum: {}", sum);
        if sum > 50 {
            set_speed(1);
        } else {
            set_speed(-1);
        }
    });

    tids.push(reader_thread);

    for i in 0..10 {
        let thread_tx = tx.clone();
        let barrier = barrier.clone();
        let tid = std::thread::spawn(move || loop {
            let v = read_value();
            println!("Sending value from thread {}: {}", i, v);
            match thread_tx.send(v) {
                Ok(_) => {}
                Err(_) => {
                    println!("Error: failed to send value");
                }
            }
            barrier.wait();
        });
        tids.push(tid);
    }

    for tid in tids {
        tid.join().expect("Error: failed to join thread");

    }
}
