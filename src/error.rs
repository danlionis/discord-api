//! Error types

use std::convert::From;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode as WsCloseCode;

#[derive(Debug)]
pub enum Error {
    DiscordError(DiscordError),
    RequestError(hyper::Error),
    ApiError(ApiError),
    ParseError(serde_json::Error),
    WebsocketError(tokio_tungstenite::tungstenite::Error),
    GatewayClosed(Option<CloseCode>),
    Custom(String),
}

#[derive(Debug)]
pub enum DiscordError {
    SendError,
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebsocketError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::ParseError(err)
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Self::RequestError(err)
    }
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        Self::ApiError(err)
    }
}

impl From<CloseCode> for Error {
    fn from(code: CloseCode) -> Self {
        Self::GatewayClosed(Some(code))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u16)]
#[allow(missing_docs)]
pub enum CloseCode {
    UnknownError = 4000,
    UnknownOpcode = 4001,
    DecodeError = 4002,
    NotAuthenticated = 4003,
    AuthenticationFailed = 4004,
    AlreadyAuthenticated = 4005,
    InvalidSeq = 4007,
    RateLimited = 4008,
    SessionTimedOut = 4009,
    InvalidShard = 4010,
    ShardingRequired = 4011,
    InvalidAPIVersion = 4012,
    InvalidIntents = 4013,
    DisallowedIntents = 4014,
}

impl From<u16> for CloseCode {
    fn from(v: u16) -> Self {
        match v {
            4001 => CloseCode::UnknownOpcode,
            4003 => CloseCode::DecodeError,
            4004 => CloseCode::NotAuthenticated,
            4005 => CloseCode::AuthenticationFailed,
            4006 => CloseCode::AlreadyAuthenticated,
            4007 => CloseCode::InvalidSeq,
            4008 => CloseCode::RateLimited,
            4009 => CloseCode::SessionTimedOut,
            4010 => CloseCode::InvalidShard,
            4011 => CloseCode::ShardingRequired,
            4012 => CloseCode::InvalidAPIVersion,
            4013 => CloseCode::InvalidIntents,
            4014 => CloseCode::DisallowedIntents,
            _ => CloseCode::UnknownError,
        }
    }
}

impl From<WsCloseCode> for CloseCode {
    fn from(v: WsCloseCode) -> Self {
        let v: u16 = v.into();
        CloseCode::from(v)
    }
}

#[derive(Debug)]
#[allow(missing_docs)]
pub enum ApiError {
    GeneralError = 0,
    UnknownAccount = 10001,
    UnknownApplication = 10002,
    UnknownChannel = 10003,
    UnknownGuild = 10004,
    UnknownIntegration = 10005,
    UnknownInvite = 10006,
    UnknownMember = 10007,
    UnknownMessage = 10008,
    UnknownPermissionOverwrite = 10009,
    UnknownProvider = 10010,
    UnknownRole = 10011,
    UnknownToken = 10012,
    UnknownUser = 10013,
    UnknownEmoji = 10014,
    UnknownWebhook = 10015,
    UnknownBan = 10026,
    UnknownSKU = 10027,
    UnknownStoreListing = 10028,
    UnknownEntitlement = 10029,
    UnknownBuild = 10030,
    UnknownLobby = 10031,
    UnknownBranch = 10032,
    UnknownRedistibutable = 10036,
    BotDenied = 20001,
    OnlyBodAllowed = 20002,
    MaxNumberOfGuilds = 30001,
    MaxNumberOfFriends = 30002,
    MaxNumberOfPins = 30003,
    MaxNumberOfRoles = 30005,
    MaxNumberOfWebhooks = 30007,
    MaxNumberOfReactions = 30010,
    MaxNumberOfChannels = 30013,
    MaxNumberOfAttachments = 30015,
    MaxNumberOfInvites = 30016,
    Unauthorized = 40001,
    AccontVerificationRequired = 40002,
    RequestTooLarge = 40005,
    TemporarilyDisables = 40006,
    Banned = 40007,
    MissingAccess = 50001,
    InvalidAccountType = 50002,
    InvalidChannelType = 50003,
    GuildWidgetDisabled = 50004,
    CannotEdit = 50005,
    EmptyMessage = 50006,
    CannotSendUser = 50007,
    CannotSendVoiceChannel = 50008,
    InsufficientChannelVerification = 50009,
    OAuth2Bot = 50010,
    OAuth2Limit = 50011,
    InvalidOAuth2 = 50012,
    InsufficientPermission = 50013,
    InvalidAuthToken = 50014,
    NoteTooLong = 50015,
    InvalidDeleteCount = 50016,
    PinMessageError = 50019,
    InvalidInvite = 50020,
    SystemMessageAction = 50021,
    InvalidOAuth2AccessToken = 50025,
    MessageTooOld = 50034,
    InvalidFormBody = 50035,
    InviteAccessFailed = 50036,
    InvalidAPIVersion = 50041,
    ReactionBlocked = 90001,
    Overloaded = 130000,
}
