use crate::rest::RestClient;

pub struct ClientBuilder {
    token: String,
}

impl ClientBuilder {
    pub fn new(token: &str) -> Self {
        ClientBuilder {
            token: token.to_owned(),
        }
    }
}

pub struct Client {
    token: String,
    rest_client: RestClient,
    ws_url: Option<String>,
    last_seq: u64,
    // event_reciever: mpsc::Receiver<Event<'a>>,
}

impl Client {
    pub fn new(token: &str) -> Self {
        Client {
            token: token.to_owned(),
            rest_client: RestClient::new(&token),
            ws_url: None,
            last_seq: 0,
        }
    }

    pub fn rest(&self) -> &RestClient {
        &self.rest_client
    }
}
