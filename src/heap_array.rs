use std::{
    iter::{ExactSizeIterator, IntoIterator, Iterator},
    slice::{from_raw_parts, from_raw_parts_mut},
    alloc::{Layout, alloc, dealloc},
    mem::{ManuallyDrop, size_of},
    ops::{Index, IndexMut},
    marker::PhantomData,
    ptr::NonNull,
    fmt::Debug
};

#[macro_export]
macro_rules! array {
    [$value:expr; $len:expr] => {{
        $crate::HeapArray::new($value, $len)
    }};
    [$($item:expr),+ $(,)?] => {{
        $crate::HeapArray::from_slice(&[$($item),+])
    }}
}

pub struct HeapArray<T> {
    ptr: NonNull<T>,
    len: usize,
    _marker: PhantomData<T>
}
impl<T: Clone> HeapArray<T> {
    #[inline]
    #[must_use]
    pub fn to_vec(&self) -> Vec<T> {
        self.as_slice().to_vec()
    }
}
impl<T: Clone> Clone for HeapArray<T> {
    fn clone(&self) -> Self {
        if self.len == 0 {
            Self {
                ptr: NonNull::dangling(),
                len: 0,
                _marker: PhantomData,
            }
        }
        else {
            let ptr = unsafe {alloc(Layout::array::<T>(self.len).unwrap())} as *mut T;
            for i in 0..self.len {
                unsafe {ptr.add(i).write(self.ptr.add(i).read().clone());}
            }
            Self {
                ptr: unsafe {NonNull::new_unchecked(ptr)},
                len: self.len,
                _marker: PhantomData
            }
        }
    }
}
impl<T: Clone> HeapArray<T> {
    pub fn from_slice(slice: &[T]) -> Self {
        let len = slice.len();
        if len == 0 {
            return Self {
                ptr: NonNull::dangling(),
                len: 0,
                _marker: PhantomData,
            };
        }
        if size_of::<T>() == 0 {
            return Self {
                ptr: NonNull::dangling(),
                len,
                _marker: PhantomData,
            };
        }
        let layout = Layout::array::<T>(len)
            .expect("array layout too large");
        let ptr = unsafe { alloc(layout) } as *mut T;
        if ptr.is_null() {panic!("Failed to allocate memory for HeapArray")}
        unsafe {
            let mut dest = ptr;
            for item in slice {
                dest.write(item.clone());
                dest = dest.add(1);
            }
        }
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            len,
            _marker: PhantomData,
        }
    }
}
impl<T> Default for HeapArray<T> {
    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            _marker: PhantomData
        }
    }
}
impl<T> HeapArray<T> {
    #[must_use]
    pub fn new(value: T, len: usize) -> Self {
        if len == 0 {
            Self {
                ptr: NonNull::dangling(),
                len: 0,
                _marker: PhantomData,
            }
        }
        else {
            if size_of::<T>() == 0 {}
            let array_ptr = unsafe {alloc(Layout::array::<T>(len).unwrap())} as *mut T;
            let value_ptr = &value as *const T;
            for i in 0..len {
                unsafe {array_ptr.add(i).write(value_ptr.read());}
            }
            Self {
                ptr: unsafe {NonNull::new_unchecked(array_ptr)},
                len,
                _marker: PhantomData
            }
        }
    }
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        unsafe {from_raw_parts(self.ptr.as_ptr(), self.len)}
    }
    #[inline]
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {from_raw_parts_mut(self.ptr.as_ptr(), self.len)}
    }
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe {Some(&*self.ptr.add(index).as_ptr())}
        } else {
            None
        }
    }
    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe {Some(&mut *self.ptr.add(index).as_ptr())}
        } else {
            None
        }
    }
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            ptr: self.ptr.as_ptr(),
            end: unsafe { self.ptr.as_ptr().add(self.len) },
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            ptr: self.ptr.as_ptr(),
            end: unsafe { self.ptr.as_ptr().add(self.len) },
            _marker: PhantomData,
        }
    }
}
impl<T> Index<usize> for HeapArray<T> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        if let Some(v) = self.get(index) {v}
        else {panic!("index out of bounds: the len is {} but the index is {}", self.len, index);}
    }
}
impl<T> IndexMut<usize> for HeapArray<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.len;
        if let Some(v) = self.get_mut(index) {v}
        else {panic!("index out of bounds: the len is {} but the index is {}", len, index);}
    }
}
impl<T> Drop for HeapArray<T> {
    fn drop(&mut self) {
        if self.len != 0 {
            unsafe {
                for i in 0..self.len {
                    self.ptr.add(i).drop_in_place();
                }
                dealloc(
                    self.ptr.as_ptr() as *mut u8,
                    Layout::array::<T>(self.len).unwrap(),
                );
            }
        }
    }
}
impl<T> IntoIterator for HeapArray<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        let len = self.len;
        IntoIter {
            buf: ManuallyDrop::new(self),
            start: 0,
            end: len,
        }
    }
}
impl<'a, T> IntoIterator for &'a HeapArray<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut HeapArray<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
impl<T: Clone> FromIterator<T> for HeapArray<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_slice(&Vec::from_iter(iter))
    }
}
impl<T: Debug> Debug for HeapArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_slice())
    }
}
unsafe impl<T: Send> Send for HeapArray<T> {}
unsafe impl<T: Sync> Sync for HeapArray<T> {}

pub struct Iter<'a, T> {
    ptr: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>,
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        } else {
            let old = self.ptr;
            self.ptr = unsafe {self.ptr.add(1)};
            Some(unsafe { &*old })
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = unsafe {self.end.offset_from(self.ptr)} as usize;
        (len, Some(len))
    }
}
impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

pub struct IterMut<'a, T> {
    ptr: *mut T,
    end: *mut T,
    _marker: PhantomData<&'a mut T>,
}
impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == self.end {
            None
        } else {
            let old = self.ptr;
            self.ptr = unsafe {self.ptr.add(1)};
            Some(unsafe { &mut *old })
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = unsafe {self.end.offset_from(self.ptr)} as usize;
        (len, Some(len))
    }
}
impl<'a, T> ExactSizeIterator for IterMut<'a, T> {}

pub struct IntoIter<T> {
    buf: ManuallyDrop<HeapArray<T>>,
    start: usize,
    end: usize,
}
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            let item = unsafe {self.buf.ptr.add(self.start).as_ptr().read()};
            self.start += 1;
            Some(item)
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end - self.start;
        (len, Some(len))
    }
}
impl<T> ExactSizeIterator for IntoIter<T> {}
impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for _ in self.by_ref() {}
    }
}