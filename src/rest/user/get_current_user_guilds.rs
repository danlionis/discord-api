use crate::{model::id::GuildId, rest::Route};
use http::{Method, Request};
use serde::Serialize;

/// Current user quild guild params
///
/// <https://discord.com/developers/docs/resources/user#get-current-user-guilds-query-string-params>
/// TODO: use this in the request function
#[derive(Debug, Serialize)]
pub struct CurrentUserGuildsParams {
    before: Option<GuildId>,
    after: Option<GuildId>,
    limit: Option<u32>,
}

/// Generate a send message request
pub fn get_current_user_guilds() -> Request<()> {
    let req = Request::builder()
        .uri(Route::CurrentUserGuilds)
        .method(Method::GET)
        .body(())
        .unwrap();

    req
}
