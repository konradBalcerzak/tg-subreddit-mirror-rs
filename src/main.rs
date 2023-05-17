mod settings;
use roux::Reddit;
use settings::Settings;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    let app_settings = Settings::new()
        .expect("Program crashed");
    let reddit_client = Reddit::new(&app_settings.reddit.client.user_agent, &app_settings.reddit.client.id, &app_settings.reddit.client.secret)
        .username(&app_settings.reddit.account.username)
        .username(&app_settings.reddit.account.password)
        .login()
        .await
        .expect("Program crashed");
    
}
