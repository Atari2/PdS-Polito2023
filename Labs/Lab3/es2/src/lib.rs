#[derive(Clone)]
pub struct MyCycle<I: Clone + Iterator> {
    iter: I,
    current_iter: I,
    times: usize,
    infinite: bool,
}

impl<I: Clone + Iterator> MyCycle<I> {
    pub fn new(iter: I, repeat: usize) -> Self {
        MyCycle {
            current_iter: iter.clone(),
            iter,
            times: repeat,
            infinite: repeat == 0,
        }
    }
}

impl<I: Clone + Iterator> Iterator for MyCycle<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.infinite || self.times > 0 {
            match self.current_iter.next() {
                Some(x) => Some(x),
                None => {
                    self.current_iter = self.iter.clone();
                    if !self.infinite {
                        self.times -= 1;
                    }
                    if self.times == 0 && !self.infinite {
                        None
                    } else {
                        self.current_iter.next()
                    }
                }
            }
        } else {
            None
        }
    }
}
