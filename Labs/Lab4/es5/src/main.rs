use std::sync::Arc;

mod cb {
    use std::sync::{Condvar, Mutex};

    #[derive(PartialEq)]
    enum BarrierState {
        Receiving,
        Releasing
    }

    struct BarrierCount {
        original: usize,
        count: usize,
        state: BarrierState
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
                mutex: Mutex::new(BarrierCount{
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

fn main() {
    let abarrier = Arc::new(cb::CyclicBarrier::new(3));
    let mut vt = Vec::new();
    for i in 0..3 {
        let cbarrier = abarrier.clone();
        vt.push(std::thread::Builder::new().name(format!("thread_{}", i)).spawn(move || {
            for j in 0..10 {
                cbarrier.wait();
                println!("after barrier {} {}\n", i, j);
            }
        }).unwrap());
    }
    for t in vt {
        t.join().unwrap();
    }
}
