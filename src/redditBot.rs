use crate::settings;
use roux::Reddit;

pub(crate) fn setup_roux(reddit_conf: &settings::RedditConf) -> Reddit {
    Reddit::new(
        reddit_conf.client.user_agent.as_str(),
        reddit_conf.client.id.as_str(),
        reddit_conf.client.secret.as_str(),
    )
}
