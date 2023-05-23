use crate::settings;
use roux::{util::RouxError, Me, Reddit};

pub(crate) async fn setup_roux(reddit_conf: &settings::RedditConf) -> Result<Me, RouxError> {
    Reddit::new(
        reddit_conf.client.user_agent.as_str(),
        reddit_conf.client.id.as_str(),
        reddit_conf.client.secret.as_str(),
    )
    .username(&reddit_conf.account.username)
    .password(&reddit_conf.account.password)
    .login()
    .await
}
