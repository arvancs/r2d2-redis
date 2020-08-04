//! Redis support for the `r2d2` connection pool.
#![doc(html_root_url = "https://docs.rs/redis_r2d2")]

pub extern crate r2d2;
pub extern crate redis;

use redis::ConnectionLike;
use std::time::Duration;

/// An `r2d2::ConnectionManager` for `redis::Client`s.
///
/// ## Example
///

/// ```
/// use std::ops::DerefMut;
/// use std::thread;
///
/// use redis_r2d2::{r2d2, redis, RedisConnectionManager};
///
/// fn main() {
///     let manager = RedisConnectionManager::new("redis://localhost").unwrap();
///     let pool = r2d2::Pool::builder()
///         .build(manager)
///         .unwrap();
///
///     let mut handles = vec![];
///
///     for _i in 0..10i32 {
///         let pool = pool.clone();
///         handles.push(thread::spawn(move || {
///             let mut conn = pool.get().unwrap();
///             let reply = redis::cmd("PING").query::<String>(conn.deref_mut()).unwrap();
///             // Alternatively, without deref():
///             let reply = redis::cmd("PING").query::<String>(&mut *conn).unwrap();
///             assert_eq!("PONG", reply);
///         }));
///     }
///
///     for h in handles {
///         h.join().unwrap();
///     }
/// }
/// ```
#[derive(Debug)]
pub struct RedisConnectionManager {
    connection_info: redis::ConnectionInfo,
    timeout: Option<Duration>,
}

impl RedisConnectionManager {
    /// Creates a new `RedisConnectionManager`.
    ///
    /// See `redis::Client::open` for a to_string of the parameter
    /// types.
    pub fn new<T: redis::IntoConnectionInfo>(
        params: T,
    ) -> Result<RedisConnectionManager, redis::RedisError> {
        RedisConnectionManager::with_timeout(params, None)
    }

    /// Creates a new `RedisConnectionManager` with connection `timeout`.
    ///
    /// See `redis::Client::open` for a to_string of the parameter
    /// types.
    pub fn with_timeout<T: redis::IntoConnectionInfo>(
        params: T,
        timeout: Option<Duration>,
    ) -> Result<RedisConnectionManager, redis::RedisError> {
        Ok(RedisConnectionManager {
            connection_info: params.into_connection_info()?,
            timeout,
        })
    }
}

impl r2d2::ManageConnection for RedisConnectionManager {
    type Connection = redis::Connection;
    type Error = redis::RedisError;

    fn connect(&self) -> Result<redis::Connection, Self::Error> {
        redis::Client::open(self.connection_info.clone()).and_then(|client| {
            if let Some(timeout) = self.timeout {
                client.get_connection_with_timeout(timeout)
            } else {
                client.get_connection()
            }
        })
    }

    fn is_valid(&self, conn: &mut redis::Connection) -> Result<(), Self::Error> {
        redis::cmd("PING").query(conn)
    }

    fn has_broken(&self, conn: &mut redis::Connection) -> bool {
        !conn.is_open()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;

    #[test]
    fn test_basic() {
        let manager = RedisConnectionManager::new("redis://localhost").unwrap();
        let pool = r2d2::Pool::builder().max_size(2).build(manager).unwrap();

        let (s1, r1) = mpsc::channel();
        let (s2, r2) = mpsc::channel();

        let pool1 = pool.clone();
        let t1 = thread::spawn(move || {
            let conn = pool1.get().unwrap();
            s1.send(()).unwrap();
            r2.recv().unwrap();
            drop(conn);
        });

        let pool2 = pool.clone();
        let t2 = thread::spawn(move || {
            let conn = pool2.get().unwrap();
            s2.send(()).unwrap();
            r1.recv().unwrap();
            drop(conn);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        pool.get().unwrap();
    }

    #[test]
    fn test_is_valid() {
        let manager = RedisConnectionManager::new("redis://localhost").unwrap();
        let pool = r2d2::Pool::builder()
            .max_size(1)
            .test_on_check_out(true)
            .build(manager)
            .unwrap();

        pool.get().unwrap();
    }
}
