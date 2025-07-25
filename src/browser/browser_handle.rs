use crate::browser::{browser_actor::BrowserActor, browser_message::BrowserMessage};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct BrowserHandle {
    sender: mpsc::Sender<BrowserMessage>,
}

impl BrowserHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = BrowserActor::new(receiver);
        tokio::spawn(actor.run());

        Self { sender }
    }

    pub async fn translate_from_google(
        &self,
        message: String,
        source: String,
        target: String,
    ) -> String {
        let (send, recv) = oneshot::channel();
        let msg = BrowserMessage::TranslateGoogle {
            respond_to: send,
            message,
            source,
            target,
        };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn translate_from_deepl(
        &self,
        message: String,
        source: String,
        target: String,
    ) -> String {
        let (send, recv) = oneshot::channel();
        let msg = BrowserMessage::TranslateDeepl {
            respond_to: send,
            message,
            source,
            target,
        };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }
}

impl Default for BrowserHandle {
    fn default() -> Self {
        Self::new()
    }
}
