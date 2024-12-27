


use crate::*;

type RedisConnResult = Result<Arc<deadpool_redis::Pool>, deadpool_redis::PoolError>;
pub async fn setupRedis() -> RedisConnResult{
    use deadpool_redis::{Config as DeadpoolRedisConfig, Runtime as DeadPoolRedisRuntime};
    let redisPassword = "geDteDd0Ltg2135FJYQ6rjNYHYkGQa70";
    let redisHost = "0.0.0.0";
    let redisPort = 6379;
    let redisUsername = "";
    let redis_conn_url = if !redisPassword.is_empty(){
        format!("redis://:{}@{}:{}", redisPassword, redisHost, redisPort)
    } else if !redisPassword.is_empty() && !redisUsername.is_empty(){
        format!("redis://{}:{}@{}:{}", redisUsername, redisPassword, redisHost, redisPort)
    } else{
        format!("redis://{}:{}", redisHost, redisPort)
    };
    let redis_pool_cfg = DeadpoolRedisConfig::from_url(&redis_conn_url);
    let redis_pool = Arc::new(redis_pool_cfg.create_pool(Some(DeadPoolRedisRuntime::Tokio1)).unwrap()); 

    Ok(redis_pool)

}