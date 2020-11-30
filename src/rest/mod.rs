mod client;
mod routes;

pub use client::RestClient;

/// create a new [`RestClient`] with default settings
pub fn client<T>(token: T) -> RestClient
where
    T: AsRef<str>,
{
    RestClient::new(token.as_ref())
}
