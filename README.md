# stupid-simple-kv

A dead-simple, pluggable, and binary-sorted key-value store for Rust.

## Features

- **Order-preserving, binary, tuple-style keys** using primitives, tuples, or
  your own struct if you implement IntoKey.
- **Pluggable API** – just use Rust types for keys and values. Memory and SQLite backends included, or
  write your own.
- **Generic value serialization:** Store any serde-serializable Rust value as a `KvValue` using serde_json.
- **Iteration & filtering:** Builder API for range/prefix queries with typed
  results.
- **Custom error types** and strict Rust interface.

## Installation

```sh
cargo add stupid-simple-kv
```

## Quickstart

```rust
use stupid_simple_kv::{Kv, MemoryBackend, KvValue};

let backend = Box::new(MemoryBackend::new());
let mut kv = Kv::new(backend);

let key = (42u64, "foo").to_key();
// automatically convert compatible value types to KvValue
kv.set(&key, "value".into())?;
let out = kv.get(&key)?;
assert_eq!(out, Some(KvValue::String("value".to_owned())));
kv.delete(&key)?;
```

## Iteration & Filtering

```rust
let backend = Box::new(MemoryBackend::new());
let mut kv = Kv::new(backend);
for id in 0..5 {
  let key = (1u64, id).to_key();
  kv.set(&key, id.into()).unwrap();
}

// List all values with prefix (1, _)
let results = kv.list().prefix(&(1u64,)).entries()?; // Vec<(KvKey, KvValue)>
```

## Custom Struct Keys

Just implement `IntoKey` for your type:

```rust
use stupid_simple_kv::IntoKey;

struct UserKey {
    namespace: String,
    id: u64,
}
impl IntoKey for UserKey {
    fn to_key(&self) -> stupid_simple_kv::KvKey {
        (&self.namespace, self.id).to_key()
    }
}
```

## SQLite backend

_Note: You can choose to not use the SQLite backend by disabling the `sqlite`
feature._

```rust
use stupid_simple_kv::{Kv, SqliteBackend};

let backend = Box::new(SqliteBackend::in_memory()?);
let mut kv = Kv::new(backend);
let key = ("foo",).to_key();
kv.set(&key, "bar".into())?;
```

## JSON Import/Export

- Easily **dump the entire key-value store to JSON** (human/debug-friendly) with
  `kv.dump_json()`.
- **Restore or initialize from JSON** using `Kv::from_json_string(...)`.
- All keys are dumped as parseable debug strings; values use a type-preserving
  JSON format.

**Example:**

```rust
let json = kv.dump_json()?;
// ...
let mut kv2 = Kv::from_json_string(&mut backend, json)?;
```

## License

MIT License © 2025 Siddharth S Singh (me@shantaram.xyz)
