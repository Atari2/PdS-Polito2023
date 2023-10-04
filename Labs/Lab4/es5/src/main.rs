use std::sync::Arc;

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
        pub fn rank(&self) -> usize {
            self.original - self.count + 1
        }
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
        pub fn last_thread(&self) -> bool {
            self.count == 1
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
        pub fn wait(&self) -> usize {
            let mut barrierqueue = self.mutex.lock().unwrap();
            if barrierqueue.queue_full() {
                // TODO: handle this
                panic!("Error: already waiting on maximum number of threads");
            }

            barrierqueue = self.cond_var.wait_while(barrierqueue, |guard| guard.releasing() && guard.spourious()).unwrap();

            let retval = barrierqueue.rank();
            if barrierqueue.last_thread() {
                // if we're the last thread to queue, release all threads
                barrierqueue.release();
                self.cond_var.notify_all();
            } else {            
                // queue the thread on wait
                barrierqueue.queue();
                barrierqueue = self.cond_var.wait_while(barrierqueue, |guard| guard.spourious()).unwrap();            // dequeue the thread
                barrierqueue.dequeue();
            }

            // if we're the last thread to dequeue, reset the barrier
            if barrierqueue.queue_empty() {
                barrierqueue.reset();
            }
            retval
        }
    }
}

fn barrier_example() {

    let abarrrier = Arc::new(cb::CyclicBarrier::new(3));

    let mut vt = Vec::new();

    for i in 0..3 {
        let cbarrier = abarrrier.clone();

        vt.push(std::thread::spawn(move || {
            for j in 0..10 {
                let rank = cbarrier.wait();
                println!("barrier open {} {}, rank {}", i, j, rank);
            }
        }));
    }

    for t in vt {
        t.join().unwrap();
    }
}

fn main() {
    barrier_example();
}
