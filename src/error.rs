//! Error types

use std::fmt::Display;
#[cfg(feature = "rest")]
use twilight_http::Error as HttpError;

/// Discord Error Types
#[derive(Debug)]
pub enum Error {
    /// Tungstenite error
    #[cfg(feature = "manager")]
    WebSocketError(tokio_tungstenite::tungstenite::Error),
    /// Reqwest error
    #[cfg(feature = "rest")]
    HttpError(HttpError),
    /// Serde parse error
    #[cfg(feature = "json")]
    ParseError(serde_json::Error),
    /// Gateway Error
    GatewayClosed(Option<CloseCode>),
    /// Custom Error
    Custom(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "manager")]
            Error::WebSocketError(err) => Display::fmt(err, f),
            #[cfg(feature = "rest")]
            Error::HttpError(err) => Display::fmt(err, f),
            #[cfg(feature = "json")]
            Error::ParseError(err) => Display::fmt(err, f),
            Error::GatewayClosed(err) => write!(f, "GatewayClosed({:?})", err),
            Error::Custom(err) => f.write_str(err),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(feature = "json")]
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::ParseError(err)
    }
}

impl From<CloseCode> for Error {
    fn from(code: CloseCode) -> Self {
        Self::GatewayClosed(Some(code))
    }
}

#[cfg(feature = "manager")]
impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocketError(err)
    }
}

#[cfg(feature = "rest")]
impl From<HttpError> for Error {
    fn from(err: HttpError) -> Self {
        Self::HttpError(err)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[allow(missing_docs)]
pub enum CloseCode {
    UnknownError,
    UnknownOpcode,
    DecodeError,
    NotAuthenticated,
    AuthenticationFailed,
    AlreadyAuthenticated,
    InvalidSeq,
    RateLimited,
    SessionTimedOut,
    InvalidShard,
    ShardingRequired,
    InvalidAPIVersion,
    InvalidIntents,
    DisallowedIntents,
    Other(u16),
}

impl Display for CloseCode {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        // TODO: implement proper display
        write!(fmt, "{:?}", self)
    }
}

impl std::error::Error for CloseCode {}

impl From<u16> for CloseCode {
    fn from(v: u16) -> Self {
        match v {
            4000 => CloseCode::UnknownError,
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
            v => CloseCode::Other(v),
        }
    }
}

impl CloseCode {
    /// Returns true if the connection can be recovered after receiving this close code
    ///
    /// <https://discord.com/developers/docs/topics/opcodes-and-status-codes#gateway-gateway-close-event-codes>
    pub fn is_recoverable(&self) -> bool {
        match self {
            CloseCode::UnknownError
            | CloseCode::UnknownOpcode
            | CloseCode::DecodeError
            | CloseCode::NotAuthenticated
            | CloseCode::AlreadyAuthenticated
            | CloseCode::InvalidSeq
            | CloseCode::RateLimited
            | CloseCode::SessionTimedOut => true,
            CloseCode::Other(code) => *code < 4000, // try to recover if the code was not a 4000 code
            _ => false,
        }
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
