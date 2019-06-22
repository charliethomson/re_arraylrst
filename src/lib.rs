mod array;
mod list;
#[cfg(test)]
mod tests {
    use {
        crate::{
            array::Array,
            list::List,
        },
        std::{
            io::{Error},
        },
    };
    #[test]
    fn iter_test() -> Result<(), Error> {
        // Tests for the from_iter function
        let l: List<usize> = List::from_iter(0..1000)?;
        
        // Tests for the into_iter function
        let mut iter = l.clone().into_iter();

        // Tests for the actual iterator
        assert_eq!(iter.next(), Some(0usize));
        assert_eq!(iter.next(), Some(1usize));
        assert_eq!(iter.next(), Some(2usize));

        Ok(())
    }

    #[test]
    fn push_pop() -> Result<(), Error> {
        let mut l: List<char> = List::with_capacity(11)?;

        l.extend(String::from("Hello World").chars());
        for c in l.clone().into_iter() {
            eprintln!("{:?}", c);
        }
        Ok(())
    }
}
