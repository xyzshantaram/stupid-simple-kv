# stupid-simple-kv

A dead-simple, pluggable, and binary-sorted key-value store for Rust.

## Features

- **FoundationDB/Deno-style keys** – type-safe, totally order-preserving keys
  using tuples, primitives, or your own struct, not macros.
- **Zero-boilerplate get/set API** – use tuple, struct, or primitive as key, no
  `.into_key()` needed!
- **Easy decoding** – get tuples/structs back using `FromKey`.
- **Ergonomic builder API** for filtered iteration:
  ```rust
  kv
    .list::<T>()
    .prefix(&("foo",))  // use tuple or struct
    .start(&("foo", 0))
    .end(&("foo", 100))
    .iter()
  ```
  Yields `(Key, T)` pairs (key and value) as Rust types.
- **In-memory backend** (`MemoryBackend`) included.
- **Optional SQLite backend** (`SqliteBackend`) via feature flag (`sqlite`).
- **Easy pluggable backends:** Implement the `KvBackend` trait for your store.
- **Store anything:** Values are serialized to `Vec<u8>` using bincode.

## Installation

```toml
[dependencies]
stupid-simple-kv = "0.1.0"

# To get the SQLite backend:
stupid-simple-kv = { version = "0.1.0", features = ["sqlite"] }
```

## Quickstart

```rust
use stupid_simple_kv::{Kv, MemoryBackend};

let mut backend = MemoryBackend::new();
let mut kv = Kv::new(backend);

// Store and fetch typed data
kv.set(("answer",), 42u32).unwrap();
let value: Option<u32> = kv.get(("answer",)).unwrap();
assert_eq!(value, Some(42));
kv.delete(("answer",)).unwrap();
```

### Iteration and Filtering

```rust
use stupid_simple_kv::{Kv, MemoryBackend};

let mut kv = Kv::new(MemoryBackend::new());
for id in 1..=3 {
    kv.set(("user", id), format!("user-{id}")).unwrap();
}

// Prefix filter:
let users: Vec<_> = kv.list::<String>()
    .prefix(&("user",)) // idiomatic filter
    .iter()
    .collect();
// users: Vec<(Key, String)>

for (key, val) in users {
    println!("key: {:?}, value: {}", key, val);
}
```

### Decoding a key into types

```rust
use stupid_simple_kv::{Key, FromKey, IntoKey};
let key = ("foo", 42u64, true).into_key();
let (namespace, id, flag): (String, u64, bool) = FromKey::from_key(&key).unwrap();
// namespace == "foo", id == 42, flag == true
```

### Custom struct keys

```rust
use stupid_simple_kv::{Key, IntoKey, FromKey, DecodeError};
struct AssetKey {
    scope: String,
    name: String,
    id: u32,
}
impl IntoKey for AssetKey {
    fn into_key(self) -> Key {
        (self.scope, self.name, self.id).into_key()
    }
}
impl FromKey for AssetKey {
    fn from_key(key: &Key) -> Result<Self, DecodeError> {
        let (scope, name, id): (String, String, u32) = FromKey::from_key(key)?;
        Ok(Self { scope, name, id })
    }
}
```

### Using SQLite backend (optional)

Enable the feature:

```toml
[dependencies]
stupid-simple-kv = { version = "0.2.0", features = ["sqlite"] }
```

Use in your code:

```rust
use stupid_simple_kv::{Kv, storages::sqlite_backend::SqliteBackend};

let mut backend = SqliteBackend::in_memory().unwrap();
let mut kv = Kv::new(backend);
kv.set(("foo",), "bar").unwrap();
```

### Custom Backends

Just implement the `KvBackend` trait for your store. See
`src/storages/kv_backend.rs`.

## License

MIT License © 2025 Siddharth S Singh (me@shantaram.xyz)
