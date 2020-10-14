macro_rules! api {
    ($e:expr) => {
        concat!("https://discord.com/api/v6", $e)
    };
    ($e:expr, $($rest:tt)*) => {
        format!(api!($e), $($rest)*)
    }
}

pub fn guilds() -> String {
    api!("/users/@me/guilds").to_owned()
}

pub fn guild(id: u64) -> String {
    api!("/guild/{}", id)
}

pub fn guild_channels(id: u64) -> String {
    api!("/guilds/{}/channels", id)
}

pub fn guild_member(guild_id: u64, user_id: u64) -> String {
    api!("/guilds/{}/members/{}", guild_id, user_id)
}

pub fn channel_messages(id: u64) -> String {
    api!("/channels/{}/messages", id)
}

pub fn text_message(id: u64, message_id: u64) -> String {
    api!("/channels/{}/messages/{}", id, message_id)
}

pub fn gateway() -> String {
    api!("/gateway").to_owned()
}
