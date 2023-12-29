//! A very mediocre hashmap

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
}

impl<K, V> MediocreMap<K, V> {
    fn hash(input: &[u8]) -> usize {
        input.iter().fold(0, |state, x| state ^ x) as usize
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

    /// Create a new Map
    pub fn new() -> Self {
        Self { lookup: vec![] }
    }

    /// Create a new map with a specified capacity for the internal vector
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            lookup: Vec::with_capacity(cap),
        }
    }

    /// Insert a given key
    pub fn insert(&mut self, key: K, value: V)
    where
        K: AsRef<[u8]>,
    {
        let index = Self::hash(&key.as_ref());

        if self.lookup.len() <= index {
            self.lookup.extend((0..=index).map(|_| None));
        }
        let mut bucket = self.lookup.get_mut(index).expect("insert broken");

        if let Some(bucket) = &mut bucket {
            bucket.push((key, Box::new(value)));
        } else {
            *bucket = Some(vec![(key, Box::new(value))]);
        }
    }

    /// Remove a given key. Returns None when the key was not present and Some(()) when the key was present
    pub fn remove(&mut self, key: &K) -> Option<()>
    where
        K: AsRef<[u8]> + PartialEq<K>,
    {
        let index = Self::hash(&key.as_ref());
        if let Some(found_index) = self.lookup.get_mut(index) {
            if let Some(item) = found_index {
                item.retain(|(k, _)| k != key);

                return Some(());
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the value at the given key
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: AsRef<[u8]> + PartialEq<K>,
    {
        let index = Self::hash(&key.as_ref());
        if let Some(found_index) = self.lookup.get(index) {
            if let Some(item) = found_index {
                let ptr = &item.iter().find(|(k, _)| k == key)?.1;

                Some(ptr)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for MediocreMap<K, V>
where
    K: AsRef<[u8]> + PartialEq<K>,
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

impl<K, V> std::ops::Index<K> for MediocreMap<K, V>
where
    K: AsRef<[u8]> + PartialEq<K>,
{
    type Output = V;
    fn index(&self, index: K) -> &Self::Output {
        return self.get(&index).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_map() -> MediocreMap<&'static str, &'static str> {
        let mut map = MediocreMap::new();
        map.insert("tk1", "tv1");
        map.insert("tk2", "tv2");
        map.insert("tk2", "tv2");
        map.insert("tk3", "tv3");
        map.insert("tk4", "tv4");
        map
    }

    #[test]
    fn test_insert() {
        let map = get_test_map();

        assert_eq!(map.get(&"tk1"), Some(&"tv1"));
        assert_eq!(map.get(&"tk2"), Some(&"tv2"));
        assert_eq!(map.get(&"tk3"), Some(&"tv3"));
        assert_eq!(map.get(&"tk4"), Some(&"tv4"));
    }

    #[test]
    fn test_remove() {
        let mut map = get_test_map();

        map.remove(&"tk1");
        map.remove(&"tk2");
        map.remove(&"tk3");
        map.remove(&"tk4");

        assert_eq!(map.get(&"tk1"), None);
        assert_eq!(map.get(&"tk2"), None);
        assert_eq!(map.get(&"tk3"), None);
        assert_eq!(map.get(&"tk4"), None);
    }

    #[test]
    fn test_iter() {
        let map = get_test_map();
        let items = map.iter().collect::<Vec<_>>();

        assert!(items.contains(&(&"tk1", &"tv1")));
        assert!(items.contains(&(&"tk2", &"tv2")));
        assert!(items.contains(&(&"tk3", &"tv3")));
        assert!(items.contains(&(&"tk4", &"tv4")));
    }

    #[test]
    fn test_index() {
        let map = get_test_map();

        assert_eq!(map["tk1"], "tv1");
        assert_eq!(map["tk2"], "tv2");
        assert_eq!(map["tk3"], "tv3");
        assert_eq!(map["tk4"], "tv4");
    }
}
