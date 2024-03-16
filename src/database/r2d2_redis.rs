extern crate num_cpus;
use dotenv::dotenv;
use r2d2;
use r2d2::PooledConnection;
use r2d2_redis::RedisConnectionManager;
use rocket::outcome::{try_outcome, Outcome};
use rocket::request::{self, FromRequest};
use rocket::{Request, State};
use std::env;
use std::ops::{Deref, DerefMut};

type Pool = r2d2::Pool<RedisConnectionManager>;
type PooledConn = PooledConnection<RedisConnectionManager>;

pub struct Conn(Option<PooledConn>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Conn {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, ()> {
        let pool = try_outcome!(req.guard::<&State<Pool>>().await);
        match pool.get() {
            Ok(database) => Outcome::Success(Conn(Some(database))),
            Err(_) => Outcome::Success(Conn(None)),
        }
    }
}
impl Deref for Conn {
    type Target = Option<PooledConn>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Conn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn init_pool() -> Pool {
    dotenv().ok();
    let redis_address = env::var("REDIS_ADDRESS").expect("REDIS_ADDRESS missing");
    let redis_port = env::var("REDIS_PORT").expect("REDIS_PORT missing");
    let manager = RedisConnectionManager::new(format!("redis://{}:{}", redis_address, redis_port))
        .expect("connection manager");
    match r2d2::Pool::builder()
        .max_size(num_cpus::get() as u32)
        .build(manager)
    {
        Ok(pool) => pool,
        Err(e) => panic!("Error: failed to create pool {}", e),
    }
}
