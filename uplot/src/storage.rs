use generic_array::{ArrayLength, GenericArray};

#[derive(Debug, Clone)]
pub struct Storage<T, N>
where
    N: ArrayLength<T>,
{
    data: GenericArray<T, N>,
    write_at: usize,
    len: usize,
}

impl<T, N> Storage<T, N>
where
    N: ArrayLength<T>,
    T: Default,
{
    pub fn new() -> Self {
        Self {
            data: Default::default(),
            write_at: 0,
            len: 0,
        }
    }
}

impl<T, N> Storage<T, N>
where
    N: ArrayLength<T>,
{
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn write(&mut self, t: T) {
        self.data[self.write_at] = t;
        self.write_at = (self.write_at + 1) % self.capacity();
        if self.len < N::USIZE {
            self.len += 1;
        }
    }

    /// Element order is unspecified
    pub fn as_slice(&self) -> &[T] {
        &self.data[..self.len]
    }
}

impl<'a, T, N> IntoIterator for &'a Storage<T, N>
where
    N: ArrayLength<T>,
    T: Copy,
{
    type Item = T;
    type IntoIter = StorageIterator<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        let index = if self.len() == self.capacity() {
            self.write_at
        } else {
            0
        };
        StorageIterator {
            s: &self,
            index,
            count: 0,
        }
    }
}

pub struct StorageIterator<'a, T, N>
where
    N: ArrayLength<T>,
    T: Copy,
{
    s: &'a Storage<T, N>,
    index: usize,
    count: usize,
}

impl<'a, T, N> Iterator for StorageIterator<'a, T, N>
where
    N: ArrayLength<T>,
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.s.len() {
            None
        } else {
            let value = self.s.data[self.index];
            self.index = (self.index + 1) % self.s.capacity();
            self.count += 1;
            Some(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use generic_array::typenum::*;

    #[test]
    fn new() {
        let s = Storage::<i8, U12>::new();
        assert_eq!(s.capacity(), U12::USIZE);
        assert_eq!(s.len(), 0);
        assert_eq!(s.as_slice(), []);
    }

    #[test]
    fn write() {
        let mut s = Storage::<i8, U4>::new();
        assert_eq!(s.len(), 0);
        assert_eq!(s.as_slice(), []);
        for t in core::i8::MIN..=core::i8::MAX {
            s.write(t);
        }
        assert_eq!(s.len(), U4::USIZE);
        assert_eq!(s.as_slice(), [124_i8, 125_i8, 126_i8, 127_i8]);
    }

    #[test]
    fn full_iterator() {
        let mut s = Storage::<i8, U4>::new();
        for t in &[1, 2, 3, 4, 5, 6] {
            s.write(*t);
        }
        assert_eq!(s.as_slice(), &[5, 6, 3, 4]);
        let a = s.into_iter().collect::<GenericArray<i8, U4>>();
        assert_eq!(&a[..], &[3, 4, 5, 6]);
    }

    #[test]
    fn not_full_iterator() {
        let mut s = Storage::<i8, U4>::new();
        for t in &[1, 2] {
            s.write(*t);
        }
        assert_eq!(s.as_slice(), &[1, 2]);
        let a = s.into_iter().collect::<GenericArray<i8, U2>>();
        assert_eq!(&a[..], &[1, 2]);
    }
}
