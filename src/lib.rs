//! A very mediocre hashmap

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    iter,
};

const TARGET_LOAD_FACTOR: f64 = 0.7;

#[cfg(test)]
mod test;

/// A very mediocre hashmap
///
/// # Examples
/// ```
/// let mut map = mediocremap::MediocreMap::new();
/// map.insert("tk1", "tv1");
/// map.insert("tk2", "tv2");
///
/// map.remove(&"tk1");
/// assert_eq!(map.get(&"tk1"), None);
/// assert_eq!(map.get(&"tk2"), Some(&"tv2"));
/// ```
#[derive(Debug, Clone)]
pub struct MediocreMap<K, V> {
    lookup: Vec<Option<Vec<(K, Box<V>)>>>,
    count: usize,
}

impl<K, V> MediocreMap<K, V> {
    fn hash(input: impl Hash) -> u64 {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }

    fn index(&self, input: impl Hash) -> usize {
        (Self::hash(input) % self.lookup.len() as u64) as usize
    }

    /// Create an iterator over all borrowed elements in the map
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.lookup
            .iter()
            .filter_map(|x| match x {
                Some(v) => Some(v.iter().collect::<Vec<_>>()),
                None => None,
            })
            .flatten()
            .map(|(k, v)| (k, v.as_ref()))
    }

    fn load_factor(&self) -> f64 {
        self.count as f64 / self.lookup.len() as f64
    }

    /// Create an iterator over all mutably borrowed elements in the map
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut K, &mut V)> {
        self.lookup
            .iter_mut()
            .filter_map(|x| match x {
                Some(v) => Some(v.iter_mut().collect::<Vec<_>>()),
                None => None,
            })
            .flatten()
            .map(|(k, v)| (k, v.as_mut()))
    }

    /// Create an iterator over all elements in the map. This consumes the map
    pub fn into_iter(self) -> impl Iterator<Item = (K, V)> {
        self.lookup
            .into_iter()
            .filter_map(|x| x)
            .flatten()
            .map(|(k, v)| (k, *v))
    }

    /// Create a new Map with the given capaity.
    /// ```
    /// let map = mediocremap::MediocreMap::<String, String>::new();
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            lookup: iter::repeat_with(|| None).take(capacity).collect(),
            count: 0,
        }
    }

    /// Create a new map with a default capacity
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    /// Gets the number of items currently stored in the hashmap
    pub fn len(&self) -> usize {
        self.count
    }

    /// Gets the number of buckets
    pub fn capacity(&self) -> usize {
        self.lookup.len()
    }

    /// Resizes the underlying store to hold `new_size` elements without cloning the data inside the map.
    pub fn resize(&mut self, new_size: usize)
    where
        K: Hash + PartialEq<K>,
    {
        let mut new_self = Self::with_capacity(new_size);

        for bucket in self.lookup.iter_mut() {
            if let Some(bucket) = bucket.take() {
                for (key, value) in bucket {
                    new_self.insert_static_boxed(key, value);
                }
            }
        }

        *self = new_self;
    }

    /// Inserts boxed value WITHOUT resizing
    fn insert_static_boxed(&mut self, key: K, value: Box<V>)
    where
        K: Hash + PartialEq<K>,
    {
        let index = self.index(&key);

        let bucket = self.lookup.get_mut(index).expect("insert broken");

        let newly_inserted = if let Some(bucket) = bucket {
            // Update the existing entry if the key already exists
            let existing = bucket.iter().enumerate().find(|(_, (k, _))| k == &key);
            if let Some((existing_idx, _)) = existing {
                let entry = bucket.get_mut(existing_idx).expect("insert broken (again)");
                *entry = (key, value);
                false
            } else {
                bucket.push((key, value));
                true
            }
        } else {
            *bucket = Some(vec![(key, value)]);
            true
        };

        if newly_inserted {
            self.count += 1;
        }
    }

    /// Inserts a `key` and `value` into the map
    ///
    /// This function allocates more space once a certain usage rate is reached
    pub fn insert(&mut self, key: K, value: V)
    where
        K: Hash + PartialEq<K>,
    {
        if self.load_factor() >= TARGET_LOAD_FACTOR {
            self.resize(self.capacity() * 2);
        }

        self.insert_static_boxed(key, Box::new(value));
    }

    /// Remove a given key. Returns None when the key was not present and it's value if it was.
    pub fn remove(&mut self, key: &K) -> Option<V>
    where
        K: Hash + PartialEq<K>,
    {
        let index = self.index(key);
        let item = self.lookup.get_mut(index)?;

        if let Some(bucket) = item {
            let (idx, _) = bucket.iter().enumerate().find(|(_, (k, _))| k == key)?;
            let (_, removed_val) = bucket.remove(idx);

            self.count -= 1;
            return Some(*removed_val);
        } else {
            None
        }
    }

    /// Get the value at the given key
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Hash + PartialEq<K>,
    {
        let index = self.index(key);
        let item = self.lookup.get(index)?;

        if let Some(bucket) = item {
            let (_, val) = &bucket.iter().find(|(k, _)| k == key)?;
            Some(val)
        } else {
            None
        }
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for MediocreMap<K, V>
where
    K: Hash + PartialEq<K>,
{
    /// Construct a map from a slice of key-value pairs
    /// # Examples
    /// ```
    /// let map = mediocremap::MediocreMap::from([("tk1", "tv1"), ("tk2", "tv2")]);
    /// assert_eq!(map.get(&"tk1"), Some(&"tv1"));
    /// assert_eq!(map.get(&"tk2"), Some(&"tv2"));
    /// ```
    fn from(value: [(K, V); N]) -> Self {
        let len = value.len();
        value
            .into_iter()
            .fold(MediocreMap::with_capacity(len), |mut state, x| {
                state.insert(x.0, x.1);
                state
            })
    }
}

impl<K, V> FromIterator<(K, V)> for MediocreMap<K, V>
where
    K: Hash + PartialEq<K>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        iter.into_iter()
            .fold(MediocreMap::with_capacity(100), |mut state, (k, v)| {
                state.insert(k, v);
                state
            })
    }
}

impl<K, V> std::ops::Index<K> for MediocreMap<K, V>
where
    K: Hash + PartialEq<K>,
{
    type Output = V;
    fn index(&self, index: K) -> &Self::Output {
        return self.get(&index).unwrap();
    }
}
