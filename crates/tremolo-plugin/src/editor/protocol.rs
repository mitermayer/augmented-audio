use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageWrapper<Message> {
    pub id: Option<String>,
    pub channel: String,
    pub message: Message,
}

impl<Message> MessageWrapper<Message> {
    pub fn new(id: Option<String>, channel: String, message: Message) -> Self {
        MessageWrapper {
            id,
            channel,
            message,
        }
    }

    pub fn notification(message: Message) -> Self {
        Self::new(None, String::from("default"), message)
    }

    pub fn request(id: &str, message: Message) -> Self {
        Self::new(None, String::from(id), message)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessageInner {
    PublishParameters(PublishParametersMessage),
}
pub type ServerMessage = MessageWrapper<ServerMessageInner>;

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishParametersMessage {
    pub parameters: Vec<ParameterDeclarationMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterDeclarationMessage {
    pub id: String,
    pub name: String,
    pub label: String,
    pub text: String,
    pub value: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessageInner {
    AppStarted(AppStartedMessage),
    SetParameter(SetParameterMessage),
    Log(LogMessage),
}

impl ClientMessageInner {
    pub fn notification(self) -> ClientMessage {
        ClientMessage::notification(self)
    }
}

pub type ClientMessage = MessageWrapper<ClientMessageInner>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppStartedMessage;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetParameterMessage {
    parameter_id: String,
    value: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogMessage {
    level: String,
    message: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serde_enum() {
        let client_message = ClientMessageInner::AppStarted(AppStartedMessage {});
        let result = serde_json::to_string(&client_message).unwrap();
        assert_eq!(result, "{\"type\":\"AppStarted\"}")
    }
}