// @generated automatically by Diesel CLI.

diesel::table! {
    channel (id) {
        id -> Nullable<Integer>,
        disabled -> Bool,
        title -> Text,
        username -> Nullable<Text>,
        invite_link -> Nullable<Text>,
    }
}

diesel::table! {
    channel_subreddit (id) {
        id -> Nullable<Integer>,
        channel_id -> Integer,
        subreddit_id -> Integer,
    }
}

diesel::table! {
    subreddit (id) {
        id -> Nullable<Integer>,
        disabled -> Bool,
        subreddit_id -> Text,
        name -> Text,
        sorting -> Text,
        post_limit -> Nullable<Integer>,
        respect_external_content_flag -> Bool,
        min_score -> Nullable<Integer>,
        allow_nsfw -> Bool,
        show_spoilers -> Bool,
        medias_only -> Bool,
    }
}

diesel::joinable!(channel_subreddit -> channel (channel_id));
diesel::joinable!(channel_subreddit -> subreddit (subreddit_id));

diesel::allow_tables_to_appear_in_same_query!(
    channel,
    channel_subreddit,
    subreddit,
);
