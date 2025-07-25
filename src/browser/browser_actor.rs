use anyhow::{Context, Result};
use fantoccini::{Client, Locator};
use std::borrow::Cow;
use tokio::sync::mpsc;

use crate::browser::browser_message::BrowserMessage;

pub struct BrowserActor {
    receiver: mpsc::Receiver<BrowserMessage>,
    client: Option<fantoccini::Client>,
}

impl BrowserActor {
    pub fn new(receiver: mpsc::Receiver<BrowserMessage>) -> Self {
        Self {
            receiver,
            client: None,
        }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
    }

    async fn client(&mut self) -> Result<&mut Client> {
        if self.client.is_none() {
            let client = fantoccini::ClientBuilder::native()
                .connect("http://localhost:53211")
                .await
                .expect("Failed to connect to browser");
            self.client = Some(client);
        }
        self.client
            .as_mut()
            .context("Unable to connect with the internet. ")
    }

    async fn handle_message(&mut self, msg: BrowserMessage) {
        match msg {
            BrowserMessage::TranslateGoogle {
                message,
                respond_to,
                source,
                target,
            } => {
                let response = match self.google(message, source, target).await {
                    Ok(value) => value,
                    Err(err) => err.to_string(),
                };
                let _ = respond_to.send(response);
            }
            BrowserMessage::TranslateDeepl {
                message,
                respond_to,
                source,
                target,
            } => {
                let response = match self.deepl(message, source, target).await {
                    Ok(value) => value,
                    Err(err) => err.to_string(),
                };
                let _ = respond_to.send(response);
            }
        }
    }

    async fn google(&mut self, text: String, source: String, target: String) -> Result<String> {
        let client = self.client().await?;

        let url = client.current_url().await?;

        let mut pairs = url.query_pairs();
        if url.domain() != Some("translate.google.com.br")
            || pairs.next() != Some((Cow::Borrowed("sl"), Cow::Borrowed(&source)))
            || pairs.next() != Some((Cow::Borrowed("tl"), Cow::Borrowed(&target)))
        {
            let _ = client
                .goto(&format!(
                    "https://deepl.com/en/translator#{source}/{target}/",
                ))
                .await;
        }

        let input = client
            .wait()
            .for_element(Locator::Css("d-textarea"))
            .await?;
        input.send_keys(&text).await?;
        let translated = client
            .wait()
            .for_element(Locator::Css(
                "[role='textbox'][aria-labelledby='translation-target-heading']",
            ))
            .await?;
        let translated_text = translated.text().await?;

        Ok(translated_text)
    }

    async fn deepl(&mut self, text: String, source: String, target: String) -> Result<String> {
        let client = self.client().await?;

        let url = client.current_url().await?;

        let mut pairs = url.query_pairs();
        if url.domain() != Some("translate.google.com.br")
            || pairs.next() != Some((Cow::Borrowed("sl"), Cow::Borrowed(&source)))
            || pairs.next() != Some((Cow::Borrowed("tl"), Cow::Borrowed(&target)))
        {
            let _ = client
                .goto(&format!(
                    "https://deepl.com/en/translator#{source}/{target}/",
                ))
                .await;
        }

        let input = client
            .wait()
            .for_element(Locator::Css("d-textarea"))
            .await?;
        input.send_keys(&text).await?;
        let translated = client
            .wait()
            .for_element(Locator::Css(
                "[role='textbox'][aria-labelledby='translation-target-heading']",
            ))
            .await?;
        let translated_text = translated.text().await?;

        Ok(translated_text)
    }
}
