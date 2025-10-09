use std::{
    iter::{Iterator, IntoIterator, ExactSizeIterator},
    slice::{from_raw_parts, from_raw_parts_mut},
    alloc::{alloc, dealloc, Layout},
    ops::{Index, IndexMut},
    marker::PhantomData,
    ptr::NonNull
};

pub struct RuntimeArray<T> {
    ptr: NonNull<T>,
    len: usize,
    _marker: PhantomData<T>
}
impl<T: Clone> RuntimeArray<T> {
    pub fn new(value: T, len: usize) -> Self {
        if len == 0 {
            Self {
                ptr: NonNull::dangling(),
                len: 0,
                _marker: PhantomData,
            }
        }
        else {
            let ptr = unsafe {alloc(Layout::array::<T>(len).unwrap())} as *mut T;
            for i in 0..len {
                unsafe {ptr.add(i).write(value.clone());}
            }
            Self {
                ptr: unsafe {NonNull::new_unchecked(ptr)},
                len,
                _marker: PhantomData
            }
        }
    }
}
impl<T> RuntimeArray<T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe {from_raw_parts(self.ptr.as_ptr(), self.len)}
    }
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {from_raw_parts_mut(self.ptr.as_ptr(), self.len)}
    }
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe {Some(&*self.ptr.add(index).as_ptr())}
        } else {
            None
        }
    }
    #[inline]
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
impl<T> Index<usize> for RuntimeArray<T> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("index out of bounds: the len is {} but the index is {}", self.len, index);
        }
        unsafe {&*self.ptr.add(index).as_ptr()}
    }
}
impl<T> IndexMut<usize> for RuntimeArray<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!("index out of bounds: the len is {} but the index is {}", self.len, index);
        }
        unsafe {&mut *self.ptr.add(index).as_ptr()}
    }
}
impl<T> Drop for RuntimeArray<T> {
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
impl<T> IntoIterator for RuntimeArray<T> {
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
impl<'a, T> IntoIterator for &'a RuntimeArray<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut RuntimeArray<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

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

use std::mem::ManuallyDrop;

pub struct IntoIter<T> {
    buf: ManuallyDrop<RuntimeArray<T>>,
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