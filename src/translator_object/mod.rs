mod imp;

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct TranslatorObject(ObjectSubclass<imp::TranslatorObject>);
}

impl TranslatorObject {
    pub fn new(code: String) -> Self {
        let land_data = TranslatorData::new(&code);
        Object::builder()
            .property("code", code)
            .property("language", land_data.language)
            .build()
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
                if t_data.code.eq(code) {
                    Some(t_data)
                } else {
                    None
                }
            });
        lang.unwrap()
    }
    pub fn all_languages() -> [TranslatorData; 11] {
        [
            TranslatorData {
                code: "auto".to_owned(),
                language: "Detect".to_owned(),
            },
            TranslatorData {
                code: "en".to_owned(),
                language: "English".to_owned(),
            },
            TranslatorData {
                code: "ar".to_owned(),
                language: "Arabic".to_owned(),
            },
            TranslatorData {
                code: "zh".to_owned(),
                language: "Chinese".to_owned(),
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
                code: "it".to_owned(),
                language: "Italian".to_owned(),
            },
            TranslatorData {
                code: "ja".to_owned(),
                language: "Japanese".to_owned(),
            },
            TranslatorData {
                code: "pt".to_owned(),
                language: "Portuguese".to_owned(),
            },
            TranslatorData {
                code: "ru".to_owned(),
                language: "Russian".to_owned(),
            },
            TranslatorData {
                code: "es".to_owned(),
                language: "Spanish".to_owned(),
            },
        ]
    }
}
