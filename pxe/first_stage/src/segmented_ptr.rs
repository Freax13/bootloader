use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[repr(C)]
pub struct SegmentedPointer<'a, T: ?Sized> {
    offset: u16,
    segment: u16,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> SegmentedPointer<'a, T> {
    pub unsafe fn new(offset: u16, segment: u16) -> Self {
        Self {
            offset,
            segment,
            _marker: PhantomData,
        }
    }

    pub fn from_mut(ptr: &'a mut T) -> Self {
        let orig_ptr = ptr as *const T;
        let ptr = ptr as *const T as u32;
        let segment = (ptr >> 4) as u16 & !0xf;
        let offset = (ptr & 0xff) as u16;
        unsafe { Self::new(offset, segment) }
    }

    pub fn offset(&self) -> u16 {
        self.offset
    }

    pub fn segment(&self) -> u16 {
        self.segment
    }

    pub fn as_ptr(&self) -> *const T {
        ((u32::from(self.segment) << 4) + u32::from(self.offset)) as *const T
    }

    pub fn as_ptr_mut(&mut self) -> *mut T {
        ((u32::from(self.segment) << 4) + u32::from(self.offset)) as *mut T
    }
}

impl<T> Deref for SegmentedPointer<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.as_ptr() }
    }
}

impl<T> DerefMut for SegmentedPointer<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.as_ptr_mut() }
    }
}
