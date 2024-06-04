mod imp;

use glib::Object;
use gtk::glib;
use headless_chrome::Browser;

use crate::{ ocr_object::OcrData, rect::Rect };

glib::wrapper! {
    pub struct TranslatorObject(ObjectSubclass<imp::TranslatorObject>);
}

impl TranslatorObject {
    pub fn new(code: String) -> Self {
        let land_data = TranslatorData::new(&code);
        Object::builder().property("code", code).property("language", land_data.language).build()
    }
}
#[derive(Default, Clone)]
pub struct TranslatorData {
    pub code: String,
    pub language: String,
}

impl TranslatorData {
    pub fn new(code: &str) -> TranslatorData {
        let lang = TranslatorData::all_languages()
            .into_iter()
            .find_map(|t_data| {
                if t_data.code.eq(code) { Some(t_data) } else { None }
            });
        lang.unwrap()
    }

    pub fn all_languages() -> [TranslatorData; 30] {
        [
            TranslatorData {
                code: "auto".to_owned(),
                language: "Detect language".to_owned(),
            },
            TranslatorData {
                code: "bg".to_owned(),
                language: "Bulgarian".to_owned(),
            },
            TranslatorData {
                code: "zh".to_owned(),
                language: "Chinese".to_owned(),
            },
            TranslatorData {
                code: "cs".to_owned(),
                language: "Czech".to_owned(),
            },
            TranslatorData {
                code: "da".to_owned(),
                language: "Danish".to_owned(),
            },
            TranslatorData {
                code: "nl".to_owned(),
                language: "Dutch".to_owned(),
            },
            TranslatorData {
                code: "en".to_owned(),
                language: "English".to_owned(),
            },
            TranslatorData {
                code: "et".to_owned(),
                language: "Estonian".to_owned(),
            },
            TranslatorData {
                code: "fi".to_owned(),
                language: "Finnish".to_owned(),
            },
            TranslatorData {
                code: "fr".to_owned(),
                language: "French".to_owned(),
            },
            TranslatorData {
                code: "de".to_owned(),
                language: "German".to_owned(),
            },
            TranslatorData {
                code: "el".to_owned(),
                language: "Greek".to_owned(),
            },
            TranslatorData {
                code: "hu".to_owned(),
                language: "Hungarian".to_owned(),
            },
            TranslatorData {
                code: "id".to_owned(),
                language: "Indonesian".to_owned(),
            },
            TranslatorData {
                code: "it".to_owned(),
                language: "Italian".to_owned(),
            },
            TranslatorData {
                code: "ja".to_owned(),
                language: "Japanese".to_owned(),
            },
            TranslatorData {
                code: "ko".to_owned(),
                language: "Korean".to_owned(),
            },
            TranslatorData {
                code: "lv".to_owned(),
                language: "Latvian".to_owned(),
            },
            TranslatorData {
                code: "lt".to_owned(),
                language: "Lithuanian".to_owned(),
            },
            TranslatorData {
                code: "nb".to_owned(),
                language: "Norwegian".to_owned(),
            },
            TranslatorData {
                code: "pl".to_owned(),
                language: "Polish".to_owned(),
            },
            TranslatorData {
                code: "pt".to_owned(),
                language: "Portuguese".to_owned(),
            },
            TranslatorData {
                code: "ro".to_owned(),
                language: "Romanian".to_owned(),
            },
            TranslatorData {
                code: "ru".to_owned(),
                language: "Russian".to_owned(),
            },
            TranslatorData {
                code: "sk".to_owned(),
                language: "Slovak".to_owned(),
            },
            TranslatorData {
                code: "sl".to_owned(),
                language: "Slovenian".to_owned(),
            },
            TranslatorData {
                code: "es".to_owned(),
                language: "Spanish".to_owned(),
            },
            TranslatorData {
                code: "sv".to_owned(),
                language: "Swedish".to_owned(),
            },
            TranslatorData {
                code: "tr".to_owned(),
                language: "Turkish".to_owned(),
            },
            TranslatorData {
                code: "uk".to_owned(),
                language: "Ukrainian".to_owned(),
            },
        ]
    }

    pub fn translate_from_ocr(
        &self,
        ocr: &OcrData,
        provider: &str,
        texts: Vec<Rect>
    ) -> Result<Vec<Rect>, anyhow::Error> {
        let mut texts = texts;
        if texts.is_empty() {
            return Ok(texts);
        }
        let text = texts
            .iter()
            .map(|rect| rect.text.clone())
            .collect::<Vec<String>>()
            .join("\n=+=\n");

        let text = self.translate(&ocr.to_translator().code, provider, &text)?;

        let _texts = text.split("\n=+=\n").collect::<Vec<&str>>();
        for (i, tx) in _texts.iter().enumerate() {
            texts[i].text = tx.to_string();
        }
        Ok(texts)
    }

    pub fn translate(
        &self,
        source: &str,
        provider: &str,
        text: &str
    ) -> Result<String, anyhow::Error> {
        match provider {
            "google" => self.translate_from_google(&self.code, source, &urlencoding::encode(text)),
            _ => self.translate_from_deepl(&self.code, source, &urlencoding::encode(text)),
        }
    }

    pub fn translate_from_deepl(
        &self,
        target: &str,
        source: &str,
        text: &str
    ) -> Result<String, anyhow::Error> {
        let path = "//*[@id=\"textareasContainer\"]/div[3]/section/div[1]/d-textarea/div";
        let url = format!(
            "https://deepl.com/en/translator#{}/{}/{}",
            source,
            target,
            text.replace(' ', "%20")
        );
        let browser = Browser::default()?;
        let tab = browser.new_tab()?;
        tab.navigate_to(&url)?;
        tab.wait_until_navigated()?;
        let mut translated_text = "".to_owned();
        for element in tab.wait_for_elements_by_xpath(path)?.iter() {
            if let Ok(txt) = element.get_inner_text() {
                translated_text.push_str(&txt);
            }
        }
        Ok(translated_text)
    }

    pub fn translate_from_google(
        &self,
        target: &str,
        source: &str,
        text: &str
    ) -> Result<String, anyhow::Error> {
        let path = "//*[@jsname=\"W297wb\"]";
        let url = format!(
            "https://translate.google.com.br/?sl={}&tl={}&text=${}&op=translate",
            source,
            target,
            text.replace(' ', "%20")
        );
        let browser = Browser::default()?;
        let tab = browser.new_tab()?;
        tab.navigate_to(&url)?;
        tab.wait_until_navigated()?;
        let mut translated_text = "".to_owned();
        for element in tab.wait_for_elements_by_xpath(path)?.iter() {
            if let Ok(txt) = element.get_inner_text() {
                translated_text.push_str(&txt);
            }
        }
        Ok(translated_text)
    }
}
