
use {
    std::{
        io::{Error, ErrorKind,},
        ptr::{read, write, copy_nonoverlapping, copy},
        alloc::{alloc_zeroed, dealloc, Layout},
        mem::{size_of},
        fmt::{Display, Debug, Formatter, Result as fmt_Result},
    },
};

pub struct Array<T> {
    ptr: *mut T,
    size: usize,
} impl<T> Array<T> {
    // Public methods

    /// Creates a new array of type T with size size
    pub fn new(size: usize) -> Result<Self, Error> {
        unsafe {
            let ptr = alloc_zeroed(Self::layout_for_size(size)?) as *mut T;
            Ok(Array {
                ptr, size
            })
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    /// Gets the value at the index, pretty self explanatory
    pub fn get(&self, index: usize) -> Result<T, Error> {
        // if the index is not less than the size of the array, it's out of bounds
        if index >= self.size {
            Err(Error::new(ErrorKind::Other, format!("ArrayErrNo 1: index {} out of range (0 -> {})", index, self.size - 1)))
        } else { unsafe {
            // *mut T.add(offset) takes the pointer and adds the offset * mem::size_of::<T> to the pointer
            // so we read the data at the index we want and return
            Ok(read(self.as_raw_ptr().add(index)))
        }}
    }

    pub fn get_slice(&self, start: usize, stop: usize, step: usize) -> Result<Self, Error> {
        // If stop is greater than the size of the array, fix that
        
        let stop = if stop > self.size { self.size } else { stop };
        // Theoretically, stop - start is the max size for a slice, but we'll shrink it after we've added all our elements
        let mut arr = Self::new(stop - start)?;
        let mut ins = 0;
        // Each index in the total range
        for index in start..stop {
            // each one that's on the step
            if index % step == 0 {
                // add it to the array
                // these unwraps are safe because index can't be out of bounds (stop is in bounds / arr has space for them)
                arr.set(ins, self.get(index).unwrap()).unwrap();
                ins += 1;
            }
        }
        arr.resize(ins)?;
        return Ok(arr);
    }

    /// Sets the value at the given index to the given value, kinda easy to understand
    /// returns the previous value from the index
    pub fn set(&mut self, index: usize, value: T) -> Result<T, Error> {
        if index >= self.size {
            return Err(Error::new(ErrorKind::Other, format!("ArrayErrNo 2: index {} out of range (0 -> {})", index, self.size - 1)));
        }
        let old = self.get(index).unwrap();
        unsafe {
            write(self.as_mut_raw_ptr().add(index), value);
        }
        Ok(old)
    }

    /// clears the array; sets all the values to 0x00
    pub fn clear(&mut self) -> Result<(), Error> {
        unsafe {
            let new_ptr = alloc_zeroed(Self::layout_for_size(self.size)?) as *mut T;
            dealloc(self.ptr as *mut u8, Self::layout_for_size(self.size)?);
            self.ptr = new_ptr;
        }
        Ok(())
    }


    /// Resizes the current array, if the array grows, it will put zeroes in the newly allocated memory,
    /// if the array shrinks, it will delete the values outside of the previously allocated memory
    pub fn resize(&mut self, new_size: usize) -> Result<(), Error> {
        unsafe {

            // allocate space for the new array
            let new_ptr = alloc_zeroed(Self::layout_for_size(new_size)?) as *mut T;
            // copy the data from the current array to the new array
            // use the smaller of the two sizes to copy over 
            copy_nonoverlapping(self.as_raw_ptr(), new_ptr, { if new_size < self.size { new_size } else { self.size } });
            // deallocate the current array
            dealloc(self.ptr as *mut u8, Self::layout_for_size(self.size)?);

            self.ptr = new_ptr;
            self.size = new_size;
        }
        Ok(())
    }

    /// Creates an Array<T> from an Iterator<T>
    pub fn from_iter<U: Iterator<Item=T>>(i: U) -> Result<Self, Error>
    where T: Clone {
        let v: Vec<T> = i.collect();
        let size = v.len();
        let mut arr = Array::new(size)?;
        for (index, item) in v.into_iter().enumerate() {
            arr.set(index, item.clone());
        }
        return Ok(arr);
    }

    pub fn shift_from(&mut self, index: usize, amt: isize) -> Result<(), Error> {
        // Copy the data from self to a buffer
        let buf = self.clone();
        // Clear out self's data
        self.clear()?;
        unsafe {
            // Write all the data from before the index from buf to self
            copy_nonoverlapping(buf.as_raw_ptr(), self.as_mut_raw_ptr(), index);
            
            // Write the data from behind the index + the amount to shift by (leaving a gap or overwriting the data that's there)
            copy_nonoverlapping(buf.as_raw_ptr().add(index), self.as_mut_raw_ptr().add({(index as isize + amt) as usize}), buf.size - index);
        }
        Ok(())
    }

    pub fn split(self, index: usize) -> Result<(Self, Self), Error> {
        let (lsize, rsize) = if index >= self.size {
            (self.size, 0)
        } else if index == 0 {
            (0, self.size)
        } else {
            (index, self.size - index) 
        };

        let mut l = Self::new(lsize)?;
        let mut r = Self::new(rsize)?;
        
        self.get_slice(0, l.size, 1)?.clone_into(&mut l)?;
        self.get_slice(l.size, self.size, 1)?.clone_into(&mut r)?;

        Ok((l, r))
    }

    /// Clones the values from `other` directly into self
    /// if there's not enough space in self for the values from other, will return an error
    /// self and other may not overlap
    pub fn clone_from(&mut self, other: &Self) -> Result<(), Error> {
        if self.size < other.size { return Err(Error::new(ErrorKind::Other, "ArrayErrNo 3: unable to clone, not enough space")) }
        unsafe {
            copy_nonoverlapping(other.as_raw_ptr(), self.as_mut_raw_ptr(), self.size);
        } 
        Ok(())
    }

    /// Clones the values from other directly to self without any checks, will only copy self.size values
    /// self and other may overlap, causing undefined behaviour
    pub unsafe fn clone_from_unchecked(&mut self, other: &Self) {
        copy(other.as_raw_ptr(), self.as_mut_raw_ptr(), self.size);
    }

    /// Clones the values from self into `other`
    /// If there's not enough space, it will return an error
    /// self and other may not overlap
    pub fn clone_into(&self, other: &mut Self) -> Result<(), Error> {
        if other.size < self.size { return Err(Error::new(ErrorKind::Other, format!("ArrayErrNo 4: Not enough space in desination to clone (self: {}, other: {})", self.size, other.size))) }
        unsafe { copy_nonoverlapping(self.as_raw_ptr(), other.as_mut_raw_ptr(), self.size); }
        Ok(())
    }

    /// Clones all values from self into other without any checks
    /// self and other may overlap, causing undefined behaviour
    pub unsafe fn clone_into_unchecked(&self, other: &mut Self) {
        copy(self.as_raw_ptr(), other.as_mut_raw_ptr(), self.size);
    }

}

// Private methods
impl<T> Array<T> {

    /// Get a Layout for an array of size `size` for type `T`.
    /// Basically a `safe` wrapper for the unstable `Layout::array<T>(usize)` function
    pub fn layout_for_size(size: usize) -> Result<Layout, Error> {
        let size_of_t = size_of::<T>();
        // set align to the size of T if it's a power of two (u32, u8, etc), otherwise, set it to the next power of two
        let align = if size_of_t.is_power_of_two() { size_of_t } else { size_of_t.next_power_of_two() };

        // use the checked from_size_align to make sure the values are correct, return an error if it does
        // ya know, just in case
        // size is the amount of elements in the array, size_of_t is the size of each element
        match Layout::from_size_align(size * size_of_t, align) {
            Ok(n) => Ok(n),
            // This (((THEORETICALLY))) isn't reachable
            Err(_) => Err(Error::new(ErrorKind::Other, format!("ArrayErrNo 5: Unable to create Layout from {{ align: {}, size: {} }}", align, size)))
        }
    }


    fn as_mut_raw_ptr(&mut self) -> *mut T {
        self.ptr as *mut T
    }

    fn as_raw_ptr(&self) -> *const T {
        self.ptr as *const T
    }
}

// Trait Implementations
impl<T: Display> Debug for Array<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt_Result {
        write!(f, "Array at: {:p} with size: {}: \n\tdata: {}", self.ptr, self.size, {
            if self.size == 0 {
                String::from("[]")
            } else {
                let mut o = String::from("[");
                for index in 0..self.size-1 {
                    o.extend(format!("{}, ", self.get(index).unwrap()).chars());
                }
                o.extend(format!("{}]", self.get(self.size-1).unwrap()).chars());
                o
            }
        })
    }
} impl<T> Clone for Array<T> {
    fn clone(&self) -> Self {
        let mut arr = Array::new(self.size).unwrap();
        arr.clone_from(&self).unwrap();
        return arr;
    }
} impl<T: Clone, U: Iterator<Item=T>> From<U> for Array<T> {
    fn from(i: U) -> Self {
        Array::from_iter(i).unwrap()
    }
} impl<T> Drop for Array<T> {
    fn drop(&mut self) {
        // eprintln!("Dropping Array at {:p}", self.ptr);
        unsafe { dealloc(self.ptr as *mut u8, Self::layout_for_size(self.size).unwrap()); }
    }
} impl<T> IntoIterator for Array<T> {
    type Item = T;
    type IntoIter = ArrayIter<Self::Item>;
    
    fn into_iter(self) -> Self::IntoIter {
        ArrayIter::new(self.as_raw_ptr(), self.size )
    }
}

pub struct ArrayIter<T> {
    arr: *const T,
    size: usize,
    cur: usize,
} impl<T> ArrayIter<T> {
    fn new(arr: *const T, size: usize) -> Self {
        ArrayIter {
            arr, size, cur: 0usize,
        }
    }
} impl<T> Iterator for ArrayIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.cur += 1;
        if self.cur == self.size {
            None
        } else { unsafe {
            Some(read(self.arr.add(self.cur-1)))
        }}
    }
}