use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct Embed {
    title: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    description: Option<String>,
    timestamp: Option<DateTime<Utc>>,
    color: Option<i32>,
    footer: Option<EmbedFooter>,
    image: Option<EmbedImage>,
    thumbnail: Option<EmbedThumbnail>,
    video: Option<EmbedVideo>,
    provider: Option<EmbedProvider>,
    author: Option<EmbedAuthor>,
    fields: Option<Vec<EmbedFields>>,
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct EmbedFooter {
    text: String,
    icon_url: Option<String>,
    proxy_icon_url: Option<String>,
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct EmbedImage {
    url: Option<String>,
    proxy_url: Option<String>,
    height: Option<i32>,
    width: Option<i32>,
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct EmbedThumbnail {
    url: Option<String>,
    proxy_url: Option<String>,
    height: Option<i32>,
    width: Option<i32>,
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct EmbedVideo {
    url: Option<String>,
    height: Option<i32>,
    width: Option<i32>,
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct EmbedProvider {
    name: Option<String>,
    url: Option<String>,
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct EmbedAuthor {
    name: Option<String>,
    url: Option<String>,
    icon_url: Option<String>,
    proxy_icon_url: Option<String>,
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct EmbedFields {
    name: String,
    value: String,
    #[serde(default)]
    inline: bool,
}
