// copy of heapless/linear_map with our Vec

use core::{fmt, mem, slice};

use crate::vec::Vec;
use core::borrow::Borrow;

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
    // 65
    pub fn clear(&mut self) {
        self.buffer.clear()
    }

    /// Returns a reference to the value corresponding to the key
    ///
    // 101
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.iter()
            .find(|&(k, _)| k.borrow() == key)
            .map(|(_, v)| v)
    }

    /// Returns a mutable reference to the value corresponding to the key
    // 127
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.iter_mut()
            .find(|&(k, _)| k.borrow() == key)
            .map(|(_, v)| v)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned.
    // 176
    pub fn insert(&mut self, key: K, mut value: V) -> Result<Option<V>, (K, V)> {
        if let Some((_, v)) = self.iter_mut().find(|&(k, _)| *k == key) {
            mem::swap(v, &mut value);
            return Ok(Some(value));
        }

        self.buffer.push_within_capacity((key, value))?;
        Ok(None)
    }

    /// An iterator visiting all key-value pairs in arbitrary order.
    ///
    // 220
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.buffer.as_slice().iter(),
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order, with mutable references to the
    /// values
    // 248
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.buffer.as_mut_slice().iter_mut(),
        }
    }

    /// An iterator visiting all keys in arbitrary order
    ///
    // 270
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(k, _)| k)
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the
    /// map
    // 289
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        let idx = self
            .keys()
            .enumerate()
            .find(|&(_, k)| k.borrow() == key)
            .map(|(idx, _)| idx);

        idx.map(|idx| self.buffer.swap_remove(idx).1)
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

//464
pub struct IterMut<'a, K, V> {
    iter: slice::IterMut<'a, (K, V)>,
}

// 468
impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|&mut (ref k, ref mut v)| (k, v))
    }
}
