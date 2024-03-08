pub use self::error::{Error, Result};
use std::{env, str::FromStr, sync::OnceLock};

mod error;

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
    })
}

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct Config {
    pub MONGO_URI: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            MONGO_URI: get_env("MONGO_URI")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::MissingEnv(name))
}

fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let value = get_env(name)?;
    value.parse::<T>().map_err(|_| Error::WrongFormat(name))
}
