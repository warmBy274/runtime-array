use std::{
    slice::{from_raw_parts, from_raw_parts_mut},
    ops::{Index, IndexMut, Deref, DerefMut},
    ptr::{null_mut, write, drop_in_place},
    alloc::{alloc, dealloc, Layout},
    any::type_name
};

pub struct Array<T: Sized> {
    ptr: *mut T,
    len: usize
}
impl<T: Sized> Array<T> {
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
impl<T: Sized + Clone> Array<T> {
    pub fn new(size: usize, default_value: T) -> Self {
        if size == 0 {
            Self {
                ptr: null_mut(),
                len: 0
            }
        }
        else {
            let raw_ptr = unsafe {alloc(Layout::array::<T>(size).expect(&format!("Failed to allocate `Array<{}>` with size {}", type_name::<T>(), size)))};
            if raw_ptr.is_null() {
                panic!("Failed to allocate `Array<{}>` with size {}", type_name::<T>(), size);
            }
            let ptr = raw_ptr as *mut T;
            for i in 0..size {
                unsafe {write(ptr.add(i), default_value.clone())};
            }
            Self {
                ptr: ptr,
                len: size
            }
        }
    }
}
impl<T: Sized> Index<usize> for Array<T> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("index out of bounds: the len is {} but the index is {}", self.len, index);
        }
        unsafe {&*self.ptr.add(index)}
    }
}
impl<T: Sized> IndexMut<usize> for Array<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!("index out of bounds: the len is {} but the index is {}", self.len, index);
        }
        unsafe {&mut *self.ptr.add(index)}
    }
}
impl<T: Sized> Drop for Array<T> {
    #[inline]
    fn drop(&mut self) {
        if self.ptr.is_null() {
            return;
        }
        for i in 0..self.len {
            unsafe {drop_in_place(self.ptr.add(i));}
        }
        unsafe {dealloc(self.ptr as *mut u8, Layout::array::<T>(self.len).unwrap());}
    }
}
impl<T: Sized> Deref for Array<T> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {from_raw_parts(self.ptr, self.len)}
    }
}

impl<T: Sized> DerefMut for Array<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {from_raw_parts_mut(self.ptr, self.len)}
    }
}