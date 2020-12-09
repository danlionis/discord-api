use std::convert::From;

#[derive(Debug)]
pub enum Error {
    DiscordError(DiscordError),
    RequestError(hyper::Error),
    ApiError(ApiError),
    ParseError(serde_json::Error),
    WebsocketError(async_tungstenite::tungstenite::Error),
    Custom(String),
}

#[derive(Debug)]
pub enum DiscordError {
    SendError,
}

impl From<async_tungstenite::tungstenite::Error> for Error {
    fn from(err: async_tungstenite::tungstenite::Error) -> Self {
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

#[derive(Debug)]
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
