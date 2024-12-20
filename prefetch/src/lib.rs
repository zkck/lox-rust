use std::usize;

pub struct Prefetched<I: Iterator, const L: usize> {
    iter: I,
    ring: [Option<I::Item>; L],
    ring_index: usize,
}

impl<I: Iterator, const L: usize> Prefetched<I, L> {
    pub fn new(iter: I) -> Self {
        let mut s = Self {
            iter,
            ring: [const { None }; L],
            ring_index: 0,
        };
        // fill ring buffer
        for _ in 0..L {
            s.next();
        }
        s
    }

    pub fn peek(&self) -> Option<&I::Item> {
        self.peek_nth(0)
    }

    pub fn peek_nth(&self, n: usize) -> Option<&I::Item> {
        if n >= L {
            None
        } else {
            self.ring[(self.ring_index + n) % L].as_ref()
        }
    }
}

impl<I: Iterator, const L: usize> Iterator for Prefetched<I, L> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let mut v = self.iter.next();
        if L != 0 {
            v = std::mem::replace(&mut self.ring[self.ring_index], v);
            self.ring_index = (self.ring_index + 1) % L;
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let array = [1, 2, 3];
        let mut peekable = Prefetched::<_, 2>::new(array.into_iter());
        assert_eq!(peekable.peek().cloned(), Some(1));
        assert_eq!(peekable.peek_nth(1).cloned(), Some(2));
        assert_eq!(peekable.peek_nth(2).cloned(), None);

        assert_eq!(peekable.next(), Some(1));
        assert_eq!(peekable.peek().cloned(), Some(2));
        assert_eq!(peekable.peek_nth(1).cloned(), Some(3));
        assert_eq!(peekable.peek_nth(2).cloned(), None);

        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.peek().cloned(), Some(3));
        assert_eq!(peekable.peek_nth(1).cloned(), None);
        assert_eq!(peekable.peek_nth(2).cloned(), None);

        assert_eq!(peekable.next(), Some(3));
        assert_eq!(peekable.peek().cloned(), None);
        assert_eq!(peekable.peek_nth(1).cloned(), None);
        assert_eq!(peekable.peek_nth(2).cloned(), None);
    }

    #[test]
    fn test_no_prefetch() {
        let array = [1, 2, 3];
        let mut peekable = Prefetched::<_, 0>::new(array.into_iter());
        assert_eq!(peekable.peek().cloned(), None);
        assert_eq!(peekable.next(), Some(1));
    }

    #[test]
    fn test_overallocated() {
        let array = [1, 2, 3];
        let mut peekable = Prefetched::<_, 5>::new(array.into_iter());
        assert_eq!(peekable.next(), Some(1));
        assert_eq!(peekable.next(), Some(2));
        assert_eq!(peekable.next(), Some(3));
        assert_eq!(peekable.peek_nth(0).cloned(), None);
        assert_eq!(peekable.peek_nth(1).cloned(), None);
        assert_eq!(peekable.peek_nth(2).cloned(), None);
    }
}
