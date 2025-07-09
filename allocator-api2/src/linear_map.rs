// copy of heapless/linear_map with our Vec

use core::{fmt, slice};

use crate::vec::Vec;

/// A fixed capacity map / dictionary that performs lookups via linear search
///
/// Note that as this map doesn't use hashing so most operations are **O(N)** instead of O(1)
///
// 8
pub struct LinearMap<K, V, const N: usize> {
    //pub(crate) buffer: Vec<(K, V), N>,
    pub(crate) buffer: crate::vec::Vec<(K, V)>,
}

// 12
impl<K, V, const N: usize> LinearMap<K, V, N> {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(N),
        }
    }
}

// 31
impl<K: Eq, V, const N: usize> LinearMap<K, V, N> {
    /// Clears the map, removing all key-value pairs
    ///
    pub fn clear(&mut self) {
        self.buffer.clear()
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    ///
    // 220
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.buffer.as_slice().iter(),
        }
    }
}

// 391
impl<K, V, const N: usize> fmt::Debug for LinearMap<K, V, N>
where
    K: Eq + fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

// 444
pub struct Iter<'a, K, V> {
    iter: slice::Iter<'a, (K, V)>,
}

// 448
impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|&(ref k, ref v)| (k, v))
    }
}
