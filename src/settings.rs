use std::env;
use config::{Config, File, Environment, ConfigError};
use serde_derive::Deserialize;

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
pub struct Settings {
    pub reddit: RedditConf,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
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
        return app_config.try_deserialize();
    }
}