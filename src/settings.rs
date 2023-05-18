use std::env;
use config::{Config, File, Environment, ConfigError};
use once_cell::sync::Lazy;
use serde_derive::Deserialize;

impl Settings {
    pub fn from_config_file() -> Result<Self, ConfigError> {
        let args = env::args();
        let args: Vec<String> = args.collect();
        let default_filename = "tg-subreddit-mirror.toml".to_owned();
        let filename = args
            .get(1)
            .unwrap_or(&default_filename);
        // Build the configuration
        let app_config = Config::builder()
            .add_source(File::with_name(filename))
            .add_source(Environment::with_prefix("tgsmrs"))
            .build()?;
        app_config.try_deserialize()
    }
}

#[derive(Deserialize, Debug)]
pub struct TeloxideConf {
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct RedditAccountConf {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct RedditClientConf {
    pub id: String,
    pub secret: String,
    pub user_agent: String,
}

#[derive(Deserialize, Debug)]
pub struct RedditConf {
    pub account: RedditAccountConf,
    pub client: RedditClientConf,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseConf {
    pub url: String,
}

pub static SETTINGS_INSTANCE: Lazy<Settings> = Lazy::new(|| Settings::from_config_file().expect("Couldn't load app configuration"));

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub teloxide: TeloxideConf,
    pub reddit: RedditConf,
    pub database: DatabaseConf
}
