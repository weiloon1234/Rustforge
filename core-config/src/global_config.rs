use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

static GLOBAL_CONFIG: OnceLock<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>> =
    OnceLock::new();

fn registry() -> &'static RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>> {
    GLOBAL_CONFIG.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Store a config singleton. Call once at boot.
/// Panics if called twice for the same type (prevents silent overwrites).
pub fn set<T: Clone + Send + Sync + 'static>(value: T) {
    let mut map = registry()
        .write()
        .unwrap_or_else(|e| e.into_inner());
    let key = TypeId::of::<T>();
    if map.contains_key(&key) {
        panic!(
            "global_config::set<{}> called twice — config singletons are write-once",
            std::any::type_name::<T>()
        );
    }
    map.insert(key, Box::new(value));
}

/// Store a config singleton. Returns Err(value) if already set.
/// Useful in tests or optional init paths.
pub fn try_set<T: Clone + Send + Sync + 'static>(value: T) -> Result<(), T> {
    let mut map = registry()
        .write()
        .unwrap_or_else(|e| e.into_inner());
    let key = TypeId::of::<T>();
    if map.contains_key(&key) {
        return Err(value);
    }
    map.insert(key, Box::new(value));
    Ok(())
}

/// Retrieve a config singleton. Returns None if not set.
pub fn get<T: Clone + Send + Sync + 'static>() -> Option<T> {
    let map = registry()
        .read()
        .unwrap_or_else(|e| e.into_inner());
    map.get(&TypeId::of::<T>())
        .and_then(|v| v.downcast_ref::<T>())
        .cloned()
}

/// Retrieve a config singleton. Panics if not set.
pub fn expect<T: Clone + Send + Sync + 'static>() -> T {
    get::<T>().unwrap_or_else(|| {
        panic!(
            "global_config::expect<{}> — not registered. Call global_config::set() at boot.",
            std::any::type_name::<T>()
        )
    })
}
