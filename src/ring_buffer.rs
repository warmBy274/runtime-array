use crate::heap_array::HeapArray;

#[macro_export]
macro_rules! ring {
    [$value:expr; $len:expr] => {{
        $crate::RingBuffer::from_slice($crate::HeapArray::new($value, $len).as_slice())
    }};
    [$($item:expr),+ $(,)?] => {{
        $crate::RingBuffer::from_slice(&[$($item),+])
    }}
}

pub struct RingBuffer<T> {
    next_read: usize,
    last_write: usize,
    data: HeapArray<T>
}
impl<T: Clone> RingBuffer<T> {
    pub fn to_vec(&mut self) -> Vec<T> {
        self.read_all().into_iter().map(|x| x.clone()).collect()
    }
}
impl<T: Clone> RingBuffer<T> {
    #[must_use]
    pub fn from_slice(slice: &[T]) -> RingBuffer<T> {
        RingBuffer {
            next_read: 0,
            last_write: slice.len() - 1,
            data: HeapArray::from_slice(slice)
        }
    }
    #[must_use]
    pub fn read(&mut self) -> Option<&T> {
        if self.next_read == (self.last_write + 1) % self.data.len() {
            None
        }
        else {
            let value = &self.data[self.next_read];
            self.next_read = (self.next_read + 1) % self.data.len();
            Some(value)
        }
    }
    pub fn write(&mut self, value: T) -> () {
        let next_last_write = (self.last_write + 1) % self.data.len();
        self.data[next_last_write] = value;
        if self.next_read == next_last_write {
            self.next_read = (self.next_read + 1) % self.data.len();
        }
    }
    #[must_use]
    pub fn read_all(&mut self) -> Vec<&T> {
        let mut result = Vec::<&T>::new();
        while self.next_read != (self.last_write + 1) % self.data.len() {
            result.push(&self.data[self.next_read]);
            self.next_read = (self.next_read + 1) % self.data.len();
        }
        result
    }
    pub fn write_all(&mut self, values: Vec<T>) -> () {
        for value in values {self.write(value);}
    }
}
unsafe impl<T: Send> Send for RingBuffer<T> {}
unsafe impl<T: Sync> Sync for RingBuffer<T> {}