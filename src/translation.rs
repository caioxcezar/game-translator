use fantoccini::elements::Element;
use fantoccini::error::NewSessionError;
use std::borrow::Cow;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{sleep, Duration, Instant};

use anyhow::{Context, Result};
use fantoccini::actions::{InputSource, KeyAction, KeyActions};
use fantoccini::key::Key;
use fantoccini::{Client, Locator};

use crate::utils;

pub async fn client() -> Result<Client, NewSessionError> {
    let mut caps = serde_json::map::Map::new();
    let opts = serde_json::json!({
        "args": ["--headless=new", "--disable-gpu"],
    });
    caps.insert("goog:chromeOptions".to_string(), opts);

    fantoccini::ClientBuilder::native()
        .capabilities(caps)
        .connect("http://localhost:50682")
        .await
}

pub async fn google(client: &Client, text: &str, source: &str, target: &str) -> Result<String> {
    let url = client.current_url().await?;

    let mut pairs = url.query_pairs();
    if url.domain() != Some("translate.google.com.br")
        || pairs.next() != Some((Cow::Borrowed("sl"), Cow::Borrowed(source)))
        || pairs.next() != Some((Cow::Borrowed("tl"), Cow::Borrowed(target)))
    {
        let _ = client
            .goto(&format!(
                "https://translate.google.com.br/?sl={source}&tl={target}&op=translate",
            ))
            .await;
    }

    let input = client.wait().for_element(Locator::Css("textarea")).await?;
    input.send_keys(text).await?;
    let translated = client
        .wait()
        .for_element(Locator::Css("[jsname='r5xl4']"))
        .await?;
    let translated_text = get_text_and_clear(client, &translated).await?;
    Ok(translated_text)
}

pub async fn deepl(client: &Client, text: &str, source: &str, target: &str) -> Result<String> {
    let url = client.current_url().await?;

    if url.domain() != Some("deepl.com") {
        let _ = client
            .goto(&format!(
                "https://deepl.com/en/translator#{source}/{target}/",
            ))
            .await;
        wait_for_full_load(client).await?;
    }

    let inputs = client.find_all(Locator::Css("d-textarea")).await?;

    let input = inputs.first().context("Can't find input to write text")?;
    let translated = inputs.get(1).context("Can't find the translated text")?;

    while !input.is_displayed().await?
        || !input.is_enabled().await?
        || !translated.is_displayed().await?
    {
        sleep(Duration::from_millis(200)).await;
    }

    input.send_keys(text).await?;
    let translated_text = get_text_and_clear(client, translated).await?;

    Ok(translated_text)
}

async fn clean_field(client: &Client) -> Result<(), fantoccini::error::CmdError> {
    let keys = KeyActions::new("keyboard".to_string())
        .then(KeyAction::Down {
            value: Key::Control.into(),
        })
        .then(KeyAction::Down { value: 'a' })
        .then(KeyAction::Up { value: 'a' })
        .then(KeyAction::Up {
            value: Key::Control.into(),
        })
        .then(KeyAction::Down {
            value: Key::Backspace.into(),
        })
        .then(KeyAction::Up {
            value: Key::Backspace.into(),
        });

    client.perform_actions(keys).await
}

async fn get_text_and_clear(
    client: &Client,
    element: &Element,
) -> Result<String, fantoccini::error::CmdError> {
    let mut current_value: String;
    loop {
        current_value = element.text().await?;
        if !current_value.trim().is_empty() {
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
    clean_field(client).await?;
    while !element.text().await?.trim().is_empty() {
        sleep(Duration::from_millis(200)).await;
    }

    Ok(current_value)
}

async fn wait_for_full_load(
    client: &fantoccini::Client,
) -> Result<(), fantoccini::error::CmdError> {
    wait_for_page_load(client).await?;
    wait_for_network_idle(client).await?;

    Ok(())
}

async fn wait_for_page_load(
    client: &fantoccini::Client,
) -> Result<(), fantoccini::error::CmdError> {
    let timeout = Duration::from_secs(30);
    let start = Instant::now();

    loop {
        let ready_state: String = client
            .execute("return document.readyState", vec![])
            .await?
            .as_str()
            .unwrap_or("")
            .to_string();

        if ready_state == "complete" {
            break;
        }

        if start.elapsed() > timeout {
            return Err(fantoccini::error::CmdError::WaitTimeout);
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Ok(())
}

async fn wait_for_network_idle(
    client: &fantoccini::Client,
) -> Result<(), fantoccini::error::CmdError> {
    let timeout = Duration::from_secs(30);
    let start = Instant::now();

    loop {
        let active_requests: i64 = client
            .execute(
                "return window.performance.getEntriesByType('resource').filter(
                r => r.responseEnd > (performance.now() - 200)
            ).length",
                vec![],
            )
            .await?
            .as_i64()
            .unwrap_or(0);

        if active_requests == 0 {
            break;
        }

        if start.elapsed() > timeout {
            return Err(fantoccini::error::CmdError::WaitTimeout);
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    Ok(())
}

fn version_url_and_name() -> Result<(String, String)> {
    let result = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86") => ("win32".to_string(), "chromedriver.exe".to_string()),
        ("windows", "x86_64") => ("win64".to_string(), "chromedriver.exe".to_string()),
        ("linux", "x86_64") => ("linux64".to_string(), "chromedriver".to_string()),
        ("macos", "x86_64") => ("mac-x64".to_string(), "chromedriver".to_string()),
        ("macos", "aarch64") => ("mac-arm64".to_string(), "chromedriver".to_string()),
        _ => {
            return Err(anyhow::Error::msg(format!(
                "Unsupported OS: {}. with the arch: {}",
                std::env::consts::OS,
                std::env::consts::ARCH
            )))
        }
    };
    Ok(result)
}

pub fn webdriver_path() -> Result<PathBuf> {
    let (_, bin_name) = version_url_and_name()?;
    let temp_dir = utils::system_path()?.join("chromedriver");
    Ok(temp_dir.join(bin_name))
}

pub fn run_webdriver(port: u32) -> Result<PathBuf> {
    let (_, bin_name) = version_url_and_name()?;
    let system_path = utils::system_path()?.join("chromedriver");
    let bin_path = system_path.join(bin_name);

    Command::new(&bin_path)
        .arg(format!("--port={port}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(bin_path)
}

pub async fn download_webdriver() -> Result<PathBuf> {
    let (arch, bin_name) = version_url_and_name()?;
    let system_dir = utils::system_path()?.join("chromedriver");
    std::fs::create_dir_all(&system_dir)?;
    let bin_path = system_dir.join(&bin_name);
    let version_name = format!("chromedriver-{arch}");

    let latest_version =
        reqwest::get("https://googlechromelabs.github.io/chrome-for-testing/LATEST_RELEASE_STABLE")
            .await?
            .text()
            .await?;
    let download_url = format!(
        "https://storage.googleapis.com/chrome-for-testing-public/{latest_version}/{arch}/{version_name}.zip"
    );

    let bytes = reqwest::get(&download_url).await?.bytes().await?;

    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let name = file.name();

        if name == format!("{version_name}/{bin_name}") {
            let mut out_file = std::fs::File::create(&bin_path)?;
            std::io::copy(&mut file, &mut out_file)?;
            break;
        }
    }
    #[cfg(unix)]
    {
        fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755)).await?;
    }

    Ok(bin_path)
}
