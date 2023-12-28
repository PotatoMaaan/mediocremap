#[derive(Debug, Clone)]
pub struct MediocreMap<K, V> {
    lookup: Vec<Option<Vec<(K, Box<V>)>>>,
}

impl<K, V> MediocreMap<K, V> {
    fn hash(input: &[u8]) -> usize {
        input.iter().fold(0, |state, x| state ^ x) as usize
    }

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

    pub fn into_iter(self) -> impl Iterator<Item = (K, V)> {
        self.lookup
            .into_iter()
            .filter_map(|x| x)
            .flatten()
            .map(|(k, v)| (k, *v))
    }

    pub fn new() -> Self {
        Self { lookup: vec![] }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            lookup: Vec::with_capacity(cap),
        }
    }

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

impl<K, V> From<&[(K, V)]> for MediocreMap<K, V>
where
    K: AsRef<[u8]> + PartialEq<K> + Clone,
    V: Clone,
{
    fn from(value: &[(K, V)]) -> Self {
        value.iter().fold(
            MediocreMap::with_capacity(value.len()),
            |mut state, (k, v)| {
                state.insert(k.clone(), v.clone());
                state
            },
        )
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
}
