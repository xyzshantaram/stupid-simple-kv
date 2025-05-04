# stupid-simple-kv

A dead-simple, extensible key-value storage library for Rust.

## Features

- Simple and transparent API.
- Drop-in memory backend (`MemoryBackend`)
- Easily extend with your own backends (disk, network, etc.).
- Stores typed data using [`bincode`](https://docs.rs/bincode/latest/bincode/).

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
stupid-simple-kv = "0.1.0"
```

## Basic Usage

```rust
use stupid_simple_kv::{Kv, MemoryBackend};

fn main() {
    let mut backend = MemoryBackend::new();
    let mut kv = Kv::new(&mut backend);

    kv.set("answer", &42u32).unwrap();
    let value: Option<u32> = kv.get("answer");
    println!("Got: {:?}", value);

    kv.delete("answer");
    assert!(kv.get::<u32, _>("answer").is_none());
}
```

### Iterating keys

```rust
for key in kv.keys_iter() {
    println!("Key: {}", key);
}
```

## Extending: Writing Your Own Backend

Implement the `KvBackend` trait for your struct:

```rust
use stupid_simple_kv::KvBackend;

struct MyBackend { /* your fields */ }

impl KvBackend for MyBackend {
    fn set(&mut self, key: String, value: Vec<u8>) { /* ... */ }
    fn get(&self, key: String) -> Option<Vec<u8>> { /* ... */ }
    fn delete(&mut self, key: String) { /* ... */ }
    fn clear(&mut self) { /* ... */ }
    fn get_many_iter<'a>(&'a self, keys: Vec<String>) -> Box<dyn Iterator<Item=Vec<u8>> + 'a> { /* ... */ }
    fn keys_iter<'a>(&'a self) -> Box<dyn Iterator<Item=String> + 'a> { /* ... */ }
}
```

Then simply pass MyBackend to Kv::new!

## License

MIT License © 2025 Siddharth S Singh (me@shantaram.xyz)
