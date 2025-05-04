# stupid-simple-kv

A dead-simple, extensible, typed, binary-sorted, key-value store for Rust.

---

## Features

- **FoundationDB/Deno-style keys** - use the key![] macro for properly-sorted,
  list-style keys
  - **Type-safe decoding** - use the `decode_key!` macro to destructure binary
    keys right back into typed Rust values
- **Ergonomic builder API** for filtered iteration:
  ```rs
  kv
    .list::<T>()
    .prefix(&key!["foo"])
    .start(&key!["foo", 0])
    .end(&key!["foo", 100])
    .iter()
  ```
  Yields `(Vec<u8>, T)` pairs (key and value).
- **In-memory backend** (`MemoryBackend`) included.
- **Optional SQLite backend** (`SqliteBackend`) via feature flag (`sqlite`).
- **Easy pluggable backends:** Implement the `KvBackend` trait for your
  preferred storage.
- **Store anything** Values are serialized to Vec&lt;u8&gt; using bincode.

---

## Installation

```toml
[dependencies]
stupid-simple-kv = "0.1.0"

# To get the SQLite backend:
[dependencies]
stupid-simple-kv = { version = "0.1.0", features = ["sqlite"] }
```

---

## Quickstart

```rust
use stupid_simple_kv::{Kv, MemoryBackend, key};

let mut backend = MemoryBackend::new();
let mut kv = Kv::new(backend);

// Store and fetch typed data
kv.set(key!["answer"], 42u32).unwrap();
let value: Option<u32> = kv.get(&key!["answer"]).unwrap();
assert_eq!(value, Some(42));
kv.delete(&key!["answer"]).unwrap();
```

---

### Iteration and Filtering

```rust
use stupid_simple_kv::{Kv, MemoryBackend, key};

let mut kv = Kv::new(MemoryBackend::new());
for id in 1..=3 {
    kv.set(key!["user", id], format!("user-{id}")).unwrap();
}

// Prefix filter:
let users: Vec<_> = kv.list::<String>()
    .prefix(&key!["user"])
    .iter()
    .collect();
// users: Vec<(Vec<u8>, String)>

for (key, val) in users {
    println!("key: {:?}, value: {}", key, val);
}
```

---

### Decoding a binary key (destructuring)

Parse the original values out of a composite key using the built-in macro:

```rust
use stupid_simple_kv::{key, decode_key};
let key = key!["foo", 42u64, true];
let (namespace, id, flag) = decode_key!((str, u64, bool), &key);
// namespace: &str == "foo", id: u64 == 42, flag: bool == true
```

You can match as many types as you support in encoding.

---

### Using SQLite backend (optional)

Enable the feature:

```toml
[dependencies]
stupid-simple-kv = { version = "0.1.0", features = ["sqlite"] }
```

Use in your code:

```rust
use stupid_simple_kv::{Kv, SqliteBackend, key};

let mut backend = SqliteBackend::in_memory().unwrap();
let mut kv = Kv::new(backend);
kv.set(key!["foo"], "bar").unwrap();
```

---

### Custom Backends

Just implement the `KvBackend` trait for your store. See
`src/storages/kv_backend.rs`.

---

## License

MIT License © 2025 Siddharth S Singh (me@shantaram.xyz)
