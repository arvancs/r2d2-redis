# redis-r2d2

[![crates.io](http://meritbadge.herokuapp.com/redis-r2d2)](https://crates.io/crates/redis-r2d2) [![Documentation](https://docs.rs/redis_r2d2/badge.svg)](https://docs.rs/redis_r2d2)

[redis-rs](https://github.com/mitsuhiko/redis-rs) support library for the [r2d2](https://github.com/sfackler/r2d2) connection pool *totally* based on [r2d2-redis](https://github.com/sorccu/r2d2-redis).

Documentation is available [here](https://docs.rs/redis_r2d2).

# Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
redis_r2d2 = "*"
```

## Standard usage

This example shows a standard use case with convenience methods provided by `redis::Commands`. You'll note that it's practically the same as if you were using the redis crate directly. Thanks to the `Deref` trait, you'll be able to call any `Connection` method directly on a pooled connection.

```rust
use std::thread;

use redis_r2d2::{r2d2, RedisConnectionManager};
use redis_r2d2::redis::Commands;

fn main() {
    let manager = RedisConnectionManager::new("redis://localhost").unwrap();
    let pool = r2d2::Pool::builder()
        .build(manager)
        .unwrap();

    let mut handles = vec![];

    for _i in 0..10i32 {
        let pool = pool.clone();
        handles.push(thread::spawn(move || {
            let mut conn = pool.get().unwrap();
            let n: i64 = conn.incr("counter", 1).unwrap();
            println!("Counter increased to {}", n);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
```

## Manual query building

Unfortunately there are cases when the `Deref` trait cannot be used. This usually happens when you need to pass the redis connection somewhere else, such as when building queries manually and/or if the redis crate doesn't expose a convenience method for a particular command (e.g. `PING`). In these cases you must use and call the `Deref` trait directly.

```rust
extern crate redis_r2d2;

use std::ops::DerefMut;
use std::thread;

use redis_r2d2::{r2d2, redis, RedisConnectionManager};

fn main() {
    let manager = RedisConnectionManager::new("redis://localhost").unwrap();
    let pool = r2d2::Pool::builder()
        .build(manager)
        .unwrap();

    let mut handles = vec![];

    for _i in 0..10i32 {
        let pool = pool.clone();
        handles.push(thread::spawn(move || {
            let mut conn = pool.get().unwrap();
            let reply = redis::cmd("PING").query::<String>(conn.deref_mut()).unwrap();
            // Alternatively, without deref():
            // let reply = redis::cmd("PING").query::<String>(&mut *conn).unwrap();
            assert_eq!("PONG", reply);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
```
