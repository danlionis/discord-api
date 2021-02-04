use crate::model::id::{ChannelId, MessageId};

struct BaseReq;

struct ChannelReq(BaseReq, ChannelId);

struct MessagesReq(ChannelReq);

struct MessagesIdReq(MessagesReq, MessageId);

impl ChannelReq {
    pub fn messages(self) -> MessagesReq {
        MessagesReq(self)
    }
}

impl MessagesReq {
    pub fn id(self, id: MessageId) -> MessagesIdReq {
        MessagesIdReq(self, id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn yeet() {
        let channel = ChannelReq(BaseReq, 0.into());
        let messages = channel.messages();
        let single_message = messages.id(0.into());
    }
}
