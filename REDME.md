# MediocreMap

A very mediocre hashmap

# Usage example

```rs
let mut map = mediocremap::MediocreMap::new();
map.insert("tk1", "tv1");
map.insert("tk2", "tv2");
map.remove(&"tk1");
assert_eq!(map.get(&"tk1"), None);
assert_eq!(map.get(&"tk2"), Some(&"tv2"));
```
