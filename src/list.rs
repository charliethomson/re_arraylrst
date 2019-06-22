use {
    crate::array::Array,
    std::{
        io::{Error, ErrorKind},
        fmt::{Display, Debug, Formatter, Result as fmt_Result},
        
    },
};


/// arr: Array<T>,
///   the crate::array:::Array<T> that the List<T> wraps
/// len: usize,
///   current number of values in the List<T>
/// cap: usize,
///   max number of values the List<T> can hold before it needs to grow
pub struct List<T> {
    arr: Array<T>,
    len: usize,
    cap: usize,
}

// Public methods
impl<T> List<T> {
    /// Tries to create a new empty list
    pub fn new() -> Result<Self, Error> {
        Ok(List {
            arr: Array::<T>::new(4)?,
            len: 0,
            cap: 4,
        })
    }

    pub fn with_capacity(cap: usize) -> Result<Self, Error> {
        Ok(List{
            arr: Array::<T>::new(cap)?,
            len: 0,
            cap,
        })
    }

    /// Creates a new list from an iterator
    pub fn from_iter<U: Iterator<Item=T>>(i: U) -> Result<Self, Error> where T: Clone {

        let (len, i) = {
            let v: Vec<T> = i.collect();
            // get the length of the iterator
            let len = v.len();
            // recreate the iterator
            let i = v.into_iter();
            (len, i)
        };
        Ok(List {
            arr: Array::<T>::from_iter(i)?,
            len: len,
            cap: len,
        })
    }

    /// Gets the value at `index`
    pub fn get(&self, index: usize) -> Result<T, Error> {
        // if the index is out of range, return an error saying that the index is out of range, do i need to explain this to you?
        if index >= self.len {
            Err(Error::new(ErrorKind::Other, "Index out of range"))
        } else {
            Ok(self.arr.get(index)?)
        }
    }

    /// Returns the length of the List<T>
    pub fn len(&self) -> usize {
        self.len
    }

    /// Pushes `value` to the front of the List<T> 
    pub fn push_front(&mut self, value: T) -> Result<(), Error> {
        self.insert(0, value)?;
        Ok(())
    }

    /// Pushes `value` to the back of the List<T> 
    pub fn push_back(&mut self, value: T) -> Result<(), Error> {
        self.insert(self.len, value)?;
        Ok(())
    }

    /// Pushes `value` to the index `index` in the List<T> 
    pub fn push(&mut self, index: usize, value: T) -> Result<(), Error> {
        self.insert(index, value)?;
        Ok(())
    }

    /// Pops and returns the value at `index`
    pub fn pop(&mut self, index: usize) -> Result<T, Error> {
        self.del(index)
    }

    /// Pops and returns the value at the back of the List<T>
    pub fn pop_back(&mut self) -> Result<T, Error> {
        self.del(self.len - 1)
    }
    
    /// Pops and returns the value at the front of the List<T>
    pub fn pop_front(&mut self) -> Result<T, Error> {
        self.del(0)
    }

}

// Private methods
impl<T> List<T> {
    /// Backend for the push methods
    /// inserts `value` at `index`
    fn insert(&mut self, index: usize, value: T) -> Result<(), Error> {
        // if the index is out of bounds, return an error
        if index > self.len() { return Err(Error::new(ErrorKind::Other, "Index out of bounds")); } 
        // if the array is full
        if self.len + 1 >= self.cap {
            // grow the array
            self.grow()?;
        }

        // shift from the index to the right one
        self.arr.shift_from(index, 1)?;
        // insert the value at the new opening
        self.arr.set(index, value)?;
        self.len += 1;
        return Ok(());
    }

    /// Backend for the pop methods
    /// removes the item at `index` and returns it
    fn del(&mut self, index: usize) -> Result<T, Error> {
        // if the index is out of bounds, return an error
        if index > self.len() { return Err(Error::new(ErrorKind::Other, "Index out of bounds")); } 

        // if the array is empty, return an error
        if self.len == 0 { return Err(Error::new(ErrorKind::Other, "List empty")); }

        // if the array should be shrunkened
        let lower_pow2 = if self.cap.is_power_of_two() { self.cap / 2 } else { self.cap.next_power_of_two() / 2 };
        if self.len < lower_pow2 {
            // shrink it
            self.shrink()?;
        }

        // get the value being popped
        let old = self.arr.get(index)?;
        // shift the array over one value, overwriting where `old` was
        self.arr.shift_from(index+1, -1)?;
        self.len -= 1;
        // return old
        return Ok(old);
    }

    /// Grows the size of the underlying array to the next power of two
    fn grow(&mut self) -> Result<(), Error> {
        self.cap = if self.cap.is_power_of_two() { self.cap * 2 } else { self.cap.next_power_of_two() };
        self.arr.resize(self.cap)?;
        Ok(())
    }

    /// Shrinks the size of the underlying array to the next power of two below, returning anything that was dropped off the end
    fn shrink(&mut self) -> Result<Self, Error> {
        // get the next power of 2 down
        let new_size = if self.cap.is_power_of_two() { self.cap / 2 } else { self.cap.next_power_of_two() / 2 };

        // get the values that'll be chopped off by the shrink
        let (a, dropped) = match self.arr.clone().split(new_size) {
            Ok((l, r)) => (l, r),
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Unable to shrink List: {}", e)))
        };

        self.arr = a;
        self.cap = new_size;

        Ok(dropped.into())
    }
}

// Trait implementations 
impl<T: Display> Debug for List<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt_Result {
        write!(f, "List {{\n\tarr: {},\n\tlen: {},\n\tcap: {}\n}};", format!("{:?}", self.arr), self.len, self.cap)
    }
} impl<T: Display> Display for List<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt_Result {
        write!(f, "{}", {
            // if the list is empty, write []
            if self.len == 0 {
                String::from("[]")
            } else {
                let mut o = String::from("[");
                // Every index except the last one
                for index in 0..self.len-1 {
                    // Put the value and a comma, for prettiness
                    // this unwrap will never be invalid because we never go above self.len
                    o.extend(format!("{}, ", self.get(index).unwrap()).chars());
                }
                // close the bracket with the last value, again this unwrap is valid
                o.extend(format!("{}]", self.get(self.len - 1).unwrap()).chars());
                o
            }
        })
    }

} impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = ListIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        ListIter::new(self)
    }

} impl<T> Extend<T> for List<T> {
    fn extend<U: IntoIterator<Item=T>>(&mut self, other: U) {
        for val in other {
            // push each value from other to the back of self
            self.push_back(val).unwrap();
        }
    }
} impl<T> Clone for List<T> {
    fn clone(&self) -> Self {
        // Clone the underlying array, carry over the other values
        List {
            arr: self.arr.clone(),
            len: self.len,
            cap: self.cap,
        }
    }
} impl<T> From<Array<T>> for List<T> {
    fn from(arr: Array<T>) -> Self {
        // grab the size off the array, because Array<T> doesn't impl Copy
        let l = arr.size();
        List {
            // keep the array, use the Array<T>'s size for len and cap
            arr: arr,
            len: l,
            cap: l,
        }
    }
} impl<T: Clone> From<Vec<T>> for List<T> {
    fn from(v: Vec<T>) -> Self {
        // Get the list from the array from the iterator from the vec
        // ezpz str8 forward amirite
        List::from(Array::from(v.into_iter()))
    }
}


pub struct ListIter<T> {
    list: List<T>,
    cur: usize,
} impl<T> ListIter<T> {
    fn new(list: List<T>) -> Self {
        ListIter {
            list: list,
            cur: 0,
        }
    }
} impl<T> Iterator for ListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.cur += 1;

        match self.list.get(self.cur - 1) {
            // if we get a value, it's legit
            Ok(n) => Some(n),
            // otherwise we got out of bounds
            Err(_) => None
        }

    }
}