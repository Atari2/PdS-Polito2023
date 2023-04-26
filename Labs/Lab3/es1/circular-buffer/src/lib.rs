// impl with default
#[cfg(feature = "withdefault")]
pub struct CircularBuffer<T: Default> {
    buffer: Vec<T>,
    read_index: usize,
    write_index: usize,
    num_elements: usize,
}

#[cfg(feature = "withoption")]
pub struct CircularBuffer<T> {
    buffer: Vec<Option<T>>,
    read_index: usize,
    write_index: usize,
    num_elements: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    EmptyBuffer,
    FullBuffer,
}

#[cfg(feature = "withoption")]
impl<T> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let mut buf = CircularBuffer { buffer: Vec::with_capacity(capacity), read_index: 0, write_index: 0, num_elements: 0 };
        for _ in 0..capacity {
            buf.buffer.push(None);
        }
        buf
    }

    pub fn write(&mut self, _element: T) -> Result<(), Error> {
        if self.num_elements == self.buffer.capacity() {
            Err(Error::FullBuffer)
        } else {
            self.buffer[self.write_index] = Some(_element);
            self.write_index = (self.write_index + 1) % self.buffer.capacity();
            self.num_elements += 1;
            Ok(())
        }
    }

    pub fn read(&mut self) -> Result<T, Error> {
        if self.num_elements == 0 {
            Err(Error::EmptyBuffer)
        } else {
            let elem = match std::mem::replace(&mut self.buffer[self.read_index], None) {
                Some(elem) => elem,
                None => return Err(Error::EmptyBuffer),
            };
            self.read_index = (self.read_index + 1) % self.buffer.capacity();
            self.num_elements -= 1;
            Ok(elem)
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.buffer.capacity() {
            self.buffer[i] = None;
        }
        self.read_index = 0;
        self.write_index = 0;
        self.num_elements = 0;
    }

    pub fn overwrite(&mut self, _element: T) {
        if self.num_elements == self.buffer.capacity() {
            self.buffer[self.write_index] = Some(_element);
            self.write_index = (self.write_index + 1) % self.buffer.capacity();
            self.read_index = (self.read_index + 1) % self.buffer.capacity();
        } else {
            self.write(_element).unwrap();
        }
    }
}

#[cfg(feature = "withdefault")]
impl<T: Default> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let mut buf = CircularBuffer { buffer: Vec::with_capacity(capacity), read_index: 0, write_index: 0, num_elements: 0 };
        for _ in 0..capacity {
            buf.buffer.push(T::default());
        }
        buf
    }

    pub fn write(&mut self, _element: T) -> Result<(), Error> {
        if self.num_elements == self.buffer.capacity() {
            Err(Error::FullBuffer)
        } else {
            self.buffer[self.write_index] = _element;
            self.write_index = (self.write_index + 1) % self.buffer.capacity();
            self.num_elements += 1;
            Ok(())
        }
    }

    pub fn read(&mut self) -> Result<T, Error> {
        if self.num_elements == 0 {
            Err(Error::EmptyBuffer)
        } else {
            let element = std::mem::take(&mut self.buffer[self.read_index]);
            self.buffer[self.read_index] = T::default();
            self.read_index = (self.read_index + 1) % self.buffer.capacity();
            self.num_elements -= 1;
            Ok(element)
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.buffer.capacity() {
            self.buffer[i] = T::default();
        }
        self.read_index = 0;
        self.write_index = 0;
        self.num_elements = 0;
    }

    pub fn overwrite(&mut self, _element: T) {
        if self.num_elements == self.buffer.capacity() {
            self.buffer[self.write_index] = _element;
            self.write_index = (self.write_index + 1) % self.buffer.capacity();
            self.read_index = (self.read_index + 1) % self.buffer.capacity();
        } else {
            self.write(_element).unwrap();
        }
    }
}
