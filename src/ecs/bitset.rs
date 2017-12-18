use std::borrow::Borrow;

const MAX_COMPONENTS: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitSet {
    bits: [u64; MAX_COMPONENTS],
}

impl BitSet {
    pub fn new() -> Self {
        BitSet { bits: [0; MAX_COMPONENTS] }
    }

    #[inline(always)]
    pub fn insert(&mut self, index: usize) {
        let (index, bit_index) = Self::split(index);
        self.bits[index] |= 1 << bit_index;
    }

    #[inline(always)]
    pub fn remove(&mut self, index: usize) {
        let (index, bit_index) = Self::split(index);
        self.bits[index] &= !(1 << bit_index);
    }

    #[inline(always)]
    pub fn contains(&self, index: usize) -> bool {
        let (index, bit_index) = Self::split(index);
        ((1 << bit_index) & self.bits[index]) > 0
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    #[inline(always)]
    pub fn intersect_with<T>(&self, rhs: T) -> Self
        where T: Borrow<Self>
    {
        let mut bs = BitSet::new();
        let rhs = rhs.borrow();
        for i in 0..MAX_COMPONENTS {
            bs.bits[i] = self.bits[i] & rhs.bits[i];
        }
        bs
    }

    #[inline(always)]
    pub fn iter(&self) -> BitSetIter {
        BitSetIter {
            bitset: *self,
            cursor: 0,
        }
    }

    #[inline(always)]
    fn split(index: usize) -> (usize, usize) {
        let len = MAX_COMPONENTS * 64;
        assert!(index < len,
                "Too many components. (MAX_COMPONENTS: {:?})",
                len);
        (index / len, index % len)
    }
}

pub struct BitSetIter {
    bitset: BitSet,
    cursor: usize,
}

impl Iterator for BitSetIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor < MAX_COMPONENTS * 64 {
            self.cursor += 1;

            if self.bitset.contains(self.cursor - 1) {
                return Some(self.cursor - 1);
            }
        }

        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicBitSet {
    bits: Vec<u64>,
}

impl DynamicBitSet {
    pub fn new() -> Self {
        DynamicBitSet { bits: Vec::new() }
    }

    #[inline(always)]
    pub fn insert(&mut self, index: usize) {
        let (index, bit_index) = Self::split(index);

        if self.bits.len() <= index {
            unsafe {
                let len = self.bits.len();
                self.bits.reserve(index + 1 - len);
                self.bits.set_len(index + 1);
            }
        }

        self.bits[index] |= 1 << bit_index;
    }

    #[inline(always)]
    pub fn remove(&mut self, index: usize) {
        let (index, bit_index) = Self::split(index);

        if self.bits.len() <= index {
            return;
        }

        self.bits[index] &= !(1 << bit_index);
    }

    #[inline(always)]
    pub fn contains(&self, index: usize) -> bool {
        let (index, bit_index) = Self::split(index);

        if self.bits.len() <= index {
            return false;
        }

        ((1 << bit_index) & self.bits[index]) > 0
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.bits.clear();
    }

    #[inline(always)]
    pub fn iter(&self) -> DynamicBitSetIter {
        DynamicBitSetIter {
            bitset: self,
            cursor: 0,
        }
    }

    #[inline(always)]
    fn split(index: usize) -> (usize, usize) {
        let len = MAX_COMPONENTS * 64;
        (index / len, index % len)
    }
}

pub struct DynamicBitSetIter<'a> {
    bitset: &'a DynamicBitSet,
    cursor: usize,
}

impl<'a> Iterator for DynamicBitSetIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.bitset.bits.len() * 64;

        while self.cursor < len {
            self.cursor += 1;

            if self.bitset.contains(self.cursor - 1) {
                return Some(self.cursor - 1);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut bits = BitSet::new();

        assert!(!bits.contains(5));

        bits.insert(5);
        assert!(bits.contains(5));

        bits.insert(9);
        assert!(bits.contains(9));
        assert!(!bits.contains(12));

        bits.insert(12);
        assert!(bits.contains(12));

        bits.insert(5);
        assert!(bits.contains(5));

        bits.insert(5);
        assert!(bits.contains(5));
        assert!(bits.contains(9));
        assert!(bits.contains(12));

        bits.remove(5);
        assert!(!bits.contains(5));
        assert!(bits.contains(9));
        assert!(bits.contains(12));

        bits.remove(12);
        assert!(!bits.contains(5));
        assert!(bits.contains(9));
        assert!(!bits.contains(12));

        bits.clear();
        assert!(bits == BitSet::new());
    }

    #[test]
    fn intersect() {
        let mut lhs = BitSet::new();
        lhs.insert(1);
        lhs.insert(3);
        lhs.insert(9);

        let mut rhs = BitSet::new();
        rhs.insert(2);
        rhs.insert(3);
        rhs.insert(10);


        let v = lhs.intersect_with(&rhs);
        assert!(!v.contains(1));
        assert!(!v.contains(2));
        assert!(v.contains(3));
        assert!(!v.contains(9));
        assert!(!v.contains(10));
    }
}