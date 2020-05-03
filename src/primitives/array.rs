use std::alloc::{AllocErr, AllocRef, Layout, LayoutErr, System, AllocInit};
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::ptr::{drop_in_place, NonNull};
use std::slice::from_raw_parts;

/// Represents the array that is heap allocated (and thus can be dynamic).
///
/// The metadata of the array is located at the start of the array.
///
/// # Data layout
///
/// | [0...8)  | [8..8 + Element Size)  | ... | [...8 + length * element size) |
/// |----------|------------------------|-----|------------------------------|
/// | Length   | First Element          | ... | Last Element                 |
///
/// # Note
///
/// Size of this array can be 0
///
#[repr(transparent)]
pub struct HeapArray<T> {
    ptr: NonNull<u8>,
    phantom: PhantomData<T>,
}

impl<T: Default> HeapArray<T> {
    #[inline]
    pub fn with_default(size: usize) -> Self {
        HeapArray::new(size, T::default)
    }
}

impl<T> HeapArray<T> {
    pub fn new(size: usize, mut init_fn: impl FnMut() -> T) -> Self {
        let ptr = allocate_memory::<T>(size).expect("Allocation error");
        unsafe {
            // write len at *ptr
            ptr.as_ptr().cast::<usize>().write(size);
            // navigate to the first element
            // SAFETY: this is safe because even if size is 0,
            // base will point *after* allocated memory, which is valid
            let base = ptr.as_ptr().cast::<usize>().add(1).cast::<T>();
            for i in 0..size {
                // The is nothing at this memory yet, so we *write* to it
                base.add(i).write(init_fn());
            }
        }
        HeapArray {
            ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        // SAFETY: safe due to the layout of the structure on the heap
        unsafe { *(self.ptr.cast::<usize>().as_ptr()) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, i: usize) -> Option<&T> {
        self.navigate_to(i).and_then(|ptr| unsafe { ptr.as_ref() })
    }

    pub fn get_mut(&mut self, i: usize) -> Option<&mut T> {
        self.navigate_to(i).and_then(|ptr| unsafe { ptr.as_mut() })
    }

    fn navigate_to(&self, offset: usize) -> Option<*mut T> {
        if offset >= self.len() {
            None
        } else {
            unsafe {
                Some(
                    self.ptr
                        .as_ptr()
                        .cast::<usize>()
                        .add(1)
                        .cast::<T>()
                        .add(offset),
                )
            }
        }
    }
}

impl<T> Drop for HeapArray<T> {
    fn drop(&mut self) {
        let data = unsafe { self.ptr.as_ptr().cast::<usize>().add(1).cast::<T>() };
        let len = self.len();
        for i in 0..len {
            unsafe { drop_in_place(data.add(i)) }
        }
        deallocate_memory::<T>(self.ptr, len);
    }
}

impl<T: Debug> Debug for HeapArray<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let len = self.len();
        // safety: safe, even if len == 0 we point 1 byte past allocation which is safe
        let data = unsafe { self.ptr.as_ptr().cast::<usize>().add(1).cast::<T>() };
        // safety: safe, either the len is 0 or the data if valid
        let data_slice = unsafe { from_raw_parts(data, len) };
        f.debug_struct("NGHeapArray")
            .field("ptr", &self.ptr)
            .field("len<in ptr>", &self.len())
            .field("data<in ptr>", &data_slice)
            .finish()
    }
}

const INDEX_MSG: &str = "Index out of bounds";

impl<T> Index<usize> for HeapArray<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect(INDEX_MSG)
    }
}

impl<T> IndexMut<usize> for HeapArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect(INDEX_MSG)
    }
}

#[inline]
fn allocate_memory<T>(n: usize) -> Result<NonNull<u8>, AllocErr> {
    let layout = get_layout::<T>(n).unwrap();
    // we ignore the size allocated as it is guarantied to be at least enough to fit n of Ts
    Ok(System.alloc(layout, AllocInit::Zeroed)?.ptr)
}

#[inline]
fn deallocate_memory<T>(ptr: NonNull<u8>, n: usize) {
    let layout = get_layout::<T>(n).unwrap();
    unsafe {
        System.dealloc(ptr, layout);
    }
}

fn get_layout<T>(n: usize) -> Result<Layout, LayoutErr> {
    let size = Layout::new::<usize>();
    let elements = Layout::array::<T>(n)?;
    Ok(size.extend(elements)?.0)
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    /// Array should fit into one stack value
    #[test]
    fn test_heap_array_size_is_usize() {
        assert_eq!(size_of::<HeapArray<usize>>(), size_of::<usize>());
        assert_eq!(size_of::<HeapArray<u8>>(), size_of::<usize>());
        assert_eq!(size_of::<HeapArray<u128>>(), size_of::<usize>());
    }

    #[test]
    fn test_array_creation() {
        let mut i = 0usize;
        const LEN: usize = 100usize;
        let h_arr = HeapArray::new(LEN, || {
            let v = i;
            i += 1;
            v
        });
        assert_eq!(h_arr.len(), LEN);
        assert_eq!(h_arr.is_empty(), false);
        for i in 0..LEN {
            assert_eq!(h_arr.get(i), Some(&i));
        }
    }

    #[test]
    fn test_get_mut() {
        const LEN: usize = 100usize;
        let mut h_arr: HeapArray<usize> = HeapArray::with_default(LEN);
        for i in 0..LEN {
            *h_arr.get_mut(i).unwrap() = 10usize;
        }
        for i in 0..LEN {
            assert_eq!(h_arr.get(i), Some(&10));
        }
    }
}
