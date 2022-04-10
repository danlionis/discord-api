use crate::rest::Route;
use http::{Method, Request};

/// Generate a send message request
pub fn get_current_user() -> Request<()> {
    let req = Request::builder()
        .uri(Route::CurrentUser)
        .method(Method::GET)
        .body(())
        .unwrap();

    req
}
