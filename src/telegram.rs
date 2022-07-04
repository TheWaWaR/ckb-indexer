use reqwest::blocking::Client;
use std::sync::{Arc, RwLock};

pub struct TelegramClient {
    token: String,
    // Forward channel message to `@username_to_id_bot` to get the chat id
    chat_id: String,
    pending_messages: Arc<RwLock<Vec<String>>>,
}

impl TelegramClient {
    pub fn new(token: String, chat_id: String) -> TelegramClient {
        let pending_messages = Arc::new(RwLock::new(Vec::with_capacity(20)));
        TelegramClient {
            token,
            chat_id,
            pending_messages,
        }
    }

    pub fn flush(&self) -> Result<String, String> {
        let (messages_len, messages_all) = {
            let pending_messages = self.pending_messages.read().unwrap();
            (pending_messages.len(), pending_messages.join("\n"))
        };
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        let client = Client::new();
        client
            .post(url)
            .query(&[
                ("chat_id", self.chat_id.as_str()),
                ("text", messages_all.as_str()),
            ])
            .send()
            .map_err(|err| err.to_string())
            .and_then(|resp| {
                if resp.status().is_success() {
                    Ok(format!("{} messages sent", messages_len))
                } else {
                    Err(format!(
                        "status: {}, text: {:?}",
                        resp.status(),
                        resp.text()
                    ))
                }
            })
    }

    pub fn send_notify(&self, message: String, buffered: bool) -> Result<String, String> {
        let messages_len = {
            let mut pending_messages = self.pending_messages.write().unwrap();
            pending_messages.push(message);
            pending_messages.len()
        };
        if !buffered || messages_len >= 20 {
            self.flush()
        } else {
            Ok("pushed".to_string())
        }
    }
}
