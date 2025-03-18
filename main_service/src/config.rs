use dotenv::dotenv;
use std::env;


#[derive(Debug)]
pub struct PostgreConfig {
    pub url: String,
}

#[derive(Debug)]
pub struct ConfigWrapper {
    pub postgre_config: PostgreConfig
}

pub fn init_config() -> ConfigWrapper{
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    ConfigWrapper { postgre_config: PostgreConfig { url: database_url } }
}