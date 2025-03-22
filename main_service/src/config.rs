use dotenv::dotenv;
use std::{env, str::FromStr};


#[derive(Debug)]
pub struct PostgreConfig {
    pub url: String,
}

#[derive(Debug)]
pub struct RabbitMQConfig {
    pub url: String,
    pub channel_pool_max:usize,
    pub channel_pool_idle_threshold_in_second: i32,
}

#[derive(Debug)]
pub struct ConfigWrapper {
    pub postgre_config: PostgreConfig,
    pub rabbitmq_config: RabbitMQConfig,
}

pub fn init_config() -> ConfigWrapper{
    dotenv().ok();

    /* postgres */
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    /* end postgres */


    /* rabbitmq */
    let rabbitmq_url = env::var("RABBITMQ_URL").expect("RABBITMQ_URL not set");
    let channel_pool_max: usize = parse_string(env::var("RABBITMQ_MAX_CHANNEL_POOL_MAX").expect("RABBITMQ_MAX_CHANNEL_POOL_MAX not set").as_str()).map_err(|e| {
        eprintln!("Error parsing RABBITMQ_MAX_CHANNEL_POOL_MAX: {}", e);
        std::process::exit(1);
    }).unwrap();
    let channel_pool_idle_threshold_in_second: i32 = parse_string(env::var("RABBITMQ_CHANNEL_POOL_IDLE_THRESHOLD_IN_SECOND").expect("RABBITMQ_CHANNEL_POOL_IDLE_THRESHOLD_IN_SECOND not set").as_str()).map_err(|e| {
        eprintln!("Error parsing RABBITMQ_CHANNEL_POOL_IDLE_THRESHOLD_IN_SECOND: {}", e);
        std::process::exit(1);
    }).unwrap();
    /* end rabbmitq */

 
    ConfigWrapper {
        postgre_config: PostgreConfig { url: database_url },
        rabbitmq_config: RabbitMQConfig { url: rabbitmq_url, channel_pool_max: channel_pool_max, channel_pool_idle_threshold_in_second: channel_pool_idle_threshold_in_second }
    }
}

fn parse_string<T: FromStr>(s: &str) -> Result<T, T::Err> {
    s.parse::<T>()
}