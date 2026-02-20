use std::collections::HashMap;
use std::hash::Hash;

/// Lightweight typed collection helpers for `Vec<T>` / slices.
/// Generated model-specific traits can build ergonomic APIs on top of this.
pub trait TypedCollectionExt<T> {
    fn pluck_typed<R>(&self, f: impl Fn(&T) -> R) -> Vec<R>;

    fn key_by_typed<K>(&self, f: impl Fn(&T) -> K) -> HashMap<K, T>
    where
        K: Eq + Hash,
        T: Clone;

    fn group_by_typed<K>(&self, f: impl Fn(&T) -> K) -> HashMap<K, Vec<T>>
    where
        K: Eq + Hash,
        T: Clone;
}

impl<T> TypedCollectionExt<T> for [T] {
    fn pluck_typed<R>(&self, f: impl Fn(&T) -> R) -> Vec<R> {
        self.iter().map(f).collect()
    }

    fn key_by_typed<K>(&self, f: impl Fn(&T) -> K) -> HashMap<K, T>
    where
        K: Eq + Hash,
        T: Clone,
    {
        let mut out = HashMap::with_capacity(self.len());
        for item in self {
            out.insert(f(item), item.clone());
        }
        out
    }

    fn group_by_typed<K>(&self, f: impl Fn(&T) -> K) -> HashMap<K, Vec<T>>
    where
        K: Eq + Hash,
        T: Clone,
    {
        let mut out = HashMap::new();
        for item in self {
            out.entry(f(item))
                .or_insert_with(Vec::new)
                .push(item.clone());
        }
        out
    }
}

impl<T> TypedCollectionExt<T> for Vec<T> {
    fn pluck_typed<R>(&self, f: impl Fn(&T) -> R) -> Vec<R> {
        self.as_slice().pluck_typed(f)
    }

    fn key_by_typed<K>(&self, f: impl Fn(&T) -> K) -> HashMap<K, T>
    where
        K: Eq + Hash,
        T: Clone,
    {
        self.as_slice().key_by_typed(f)
    }

    fn group_by_typed<K>(&self, f: impl Fn(&T) -> K) -> HashMap<K, Vec<T>>
    where
        K: Eq + Hash,
        T: Clone,
    {
        self.as_slice().group_by_typed(f)
    }
}

#[cfg(test)]
mod tests {
    use super::TypedCollectionExt;

    #[test]
    fn typed_collection_helpers_work() {
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct Item {
            id: i64,
            kind: &'static str,
        }
        let items = vec![
            Item { id: 1, kind: "a" },
            Item { id: 2, kind: "b" },
            Item { id: 3, kind: "a" },
        ];

        let ids = items.pluck_typed(|i| i.id);
        assert_eq!(ids, vec![1, 2, 3]);

        let keyed = items.key_by_typed(|i| i.id);
        assert_eq!(keyed.get(&2).map(|i| i.kind), Some("b"));

        let grouped = items.group_by_typed(|i| i.kind);
        assert_eq!(grouped.get("a").map(|v| v.len()), Some(2));
        assert_eq!(grouped.get("b").map(|v| v.len()), Some(1));
    }
}
