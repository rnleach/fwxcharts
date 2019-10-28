use crate::sources::StringData;
use bufkit_data::BufkitDataErr;

pub struct Message(InnerMessage);

impl Message {
    pub(crate) fn payload(self) -> InnerMessage {
        self.0
    }
}

impl From<InnerMessage> for Message {
    fn from(inner_message: InnerMessage) -> Self {
        Message(inner_message)
    }
}

pub(crate) enum InnerMessage {
    StringData(StringData),
    BufkitDataError(BufkitDataErr),
}
