use tokio::sync::oneshot;

pub enum BrowserMessage {
    TranslateGoogle {
        message: String,
        respond_to: oneshot::Sender<String>,
        source: String,
        target: String,
    },
    TranslateDeepl {
        message: String,
        respond_to: oneshot::Sender<String>,
        source: String,
        target: String,
    },
}
