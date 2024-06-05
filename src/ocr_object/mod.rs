mod imp;

use glib::Object;
use gtk::glib;
use rusty_tesseract::Image;
use crate::{ rect::Rect, screen_object::ScreenData };
use rayon::prelude::*;

use crate::translator_object::TranslatorData;

glib::wrapper! {
    pub struct OcrObject(ObjectSubclass<imp::OcrObject>);
}

impl OcrObject {
    pub fn new(code: String) -> Self {
        let land_data = OcrData::new(&code);
        Object::builder().property("code", code).property("language", land_data.language).build()
    }
}
#[derive(Default, Clone)]
pub struct OcrData {
    pub code: String,
    pub language: String,
}

impl OcrData {
    pub fn to_translator(&self) -> TranslatorData {
        let code = match self.code.as_str() {
            "eng" => "en",
            "nld" => "nl",
            "dan" => "da",
            "ces" => "cs",
            "chi_sim" => "zh",
            "bul" => "bg",
            "est" => "et",
            "fin" => "fi",
            "fra" => "fr",
            "deu" => "de",
            "ell" => "el",
            "hun" => "hu",
            "ind" => "id",
            "ita" => "it",
            "jpn" => "ja",
            "kor" => "ko",
            "lav" => "lv",
            "lit" => "lt",
            "nor" => "nb",
            "pol" => "pl",
            "por" => "pt",
            "ron" => "ro",
            "rus" => "ru",
            "slk" => "sk",
            "slv" => "sl",
            "spa" => "es",
            "swe" => "sv",
            "tur" => "tr",
            "ukr" => "uk",
            _ => "auto",
        };
        TranslatorData::new(code)
    }

    pub fn new(code: &str) -> OcrData {
        let language: &str = match code {
            "afr" => "Afrikaans",
            "amh" => "Amharic",
            "ara" => "Arabic",
            "asm" => "Assamese",
            "aze" => "Azerbaijani",
            "aze_cyrl" => "Azerbaijani - Cyrilic",
            "bel" => "Belarusian",
            "ben" => "Bengali",
            "bod" => "Tibetan",
            "bos" => "Bosnian",
            "bre" => "Breton",
            "bul" => "Bulgarian",
            "cat" => "Catalan; Valencian",
            "ceb" => "Cebuano",
            "ces" => "Czech",
            "chi_sim" => "Chinese - Simplified",
            "chi_tra" => "Chinese - Traditional",
            "chr" => "Cherokee",
            "cos" => "Corsican",
            "cym" => "Welsh",
            "dan" => "Danish",
            "dan_frak" => "Danish - Fraktur (contrib)",
            "deu" => "German",
            "deu_frak" => "German - Fraktur (contrib)",
            "dzo" => "Dzongkha",
            "ell" => "Greek, Modern (1453-)",
            "eng" => "English",
            "enm" => "English, Middle (1100-1500)",
            "epo" => "Esperanto",
            "equ" => "Math / equation detection module",
            "est" => "Estonian",
            "eus" => "Basque",
            "fao" => "Faroese",
            "fas" => "Persian",
            "fil" => "Filipino (old - Tagalog)",
            "fin" => "Finnish",
            "fra" => "French",
            "frk" => "German - Fraktur",
            "frm" => "French, Middle (ca.1400-1600)",
            "fry" => "Western Frisian",
            "gla" => "Scottish Gaelic",
            "gle" => "Irish",
            "glg" => "Galician",
            "grc" => "Greek, Ancient (to 1453) (contrib)",
            "guj" => "Gujarati",
            "hat" => "Haitian; Haitian Creole",
            "heb" => "Hebrew",
            "hin" => "Hindi",
            "hrv" => "Croatian",
            "hun" => "Hungarian",
            "hye" => "Armenian",
            "iku" => "Inuktitut",
            "ind" => "Indonesian",
            "isl" => "Icelandic",
            "ita" => "Italian",
            "ita_old" => "Italian - Old",
            "jav" => "Javanese",
            "jpn" => "Japanese",
            "jpn_vert" => "Japanese Vertical",
            "kan" => "Kannada",
            "kat" => "Georgian",
            "kat_old" => "Georgian - Old",
            "kaz" => "Kazakh",
            "khm" => "Central Khmer",
            "kir" => "Kirghiz; Kyrgyz",
            "kmr" => "Kurmanji (Kurdish - Latin Script)",
            "kor" => "Korean",
            "kor_vert" => "Korean (vertical)",
            "kur" => "Kurdish (Arabic Script)",
            "lao" => "Lao",
            "lat" => "Latin",
            "lav" => "Latvian",
            "lit" => "Lithuanian",
            "ltz" => "Luxembourgish",
            "mal" => "Malayalam",
            "mar" => "Marathi",
            "mkd" => "Macedonian",
            "mlt" => "Maltese",
            "mon" => "Mongolian",
            "mri" => "Maori",
            "msa" => "Malay",
            "mya" => "Burmese",
            "nep" => "Nepali",
            "nld" => "Dutch; Flemish",
            "nor" => "Norwegian",
            "oci" => "Occitan (post 1500)",
            "ori" => "Oriya",
            "osd" => "Orientation and script detection module",
            "pan" => "Panjabi; Punjabi",
            "pol" => "Polish",
            "por" => "Portuguese",
            "pus" => "Pushto; Pashto",
            "que" => "Quechua",
            "ron" => "Romanian; Moldavian; Moldovan",
            "rus" => "Russian",
            "san" => "Sanskrit",
            "sin" => "Sinhala; Sinhalese",
            "slk" => "Slovak",
            "slk_frak" => "Slovak - Fraktur (contrib)",
            "slv" => "Slovenian",
            "snd" => "Sindhi",
            "spa" => "Spanish; Castilian",
            "spa_old" => "Spanish; Castilian - Old",
            "sqi" => "Albanian",
            "srp" => "Serbian",
            "srp_latn" => "Serbian - Latin",
            "sun" => "Sundanese",
            "swa" => "Swahili",
            "swe" => "Swedish",
            "syr" => "Syriac",
            "tam" => "Tamil",
            "tat" => "Tatar",
            "tel" => "Telugu",
            "tgk" => "Tajik",
            "tgl" => "Tagalog (new - Filipino)",
            "tha" => "Thai",
            "tir" => "Tigrinya",
            "ton" => "Tonga",
            "tur" => "Turkish",
            "uig" => "Uighur; Uyghur",
            "ukr" => "Ukrainian",
            "urd" => "Urdu",
            "uzb" => "Uzbek",
            "uzb_cyrl" => "Uzbek - Cyrilic",
            "vie" => "Vietnamese",
            "yid" => "Yiddish",
            "yor" => "Yoruba",
            _ => "Invalid",
        };
        OcrData {
            code: code.to_owned(),
            language: language.to_owned(),
        }
    }

    pub fn ocr_areas(
        &self,
        areas: &Vec<Rect>,
        screen: &ScreenData
    ) -> Result<Vec<Rect>, anyhow::Error> {
        let default_args = rusty_tesseract::Args {
            lang: self.code.to_owned(),
            ..Default::default()
        };
        let rects = areas
            .par_iter()
            .flat_map(
                |area| -> Result<Rect, anyhow::Error> {
                    let result = self.ocr_area(area, &default_args, screen);
                    if result.is_err() {
                        println!("Error: {:?}", &result);
                    }
                    result
                }
            )
            .collect::<Vec<Rect>>();
        Ok(rects)
    }

    pub fn ocr_area(
        &self,
        area: &Rect,
        default_args: &rusty_tesseract::Args,
        screen: &ScreenData
    ) -> Result<Rect, anyhow::Error> {
        let path = screen.capture_area(
            area.x as u32,
            area.y as u32,
            area.width as u32,
            area.height as u32
        )?;
        let image = Image::from_path(&path)?;
        let text = rusty_tesseract::image_to_string(&image, default_args)?.trim().to_string();
        screen.remove(&path)?;
        Ok(Rect { text, ..area.clone() })
    }

    pub fn ocr_screen(&self, screen: &ScreenData) -> Result<Vec<Rect>, anyhow::Error> {
        let default_args = rusty_tesseract::Args {
            lang: self.code.to_owned(),
            ..Default::default()
        };
        let path = screen.capture()?;
        let image = Image::from_path(&path)?;
        let output = rusty_tesseract::image_to_data(&image, &default_args)?;
        // screen.remove(&path)?;
        let mut texts = Vec::new();
        let mut line: Rect = Default::default();
        for dt in output.data {
            if dt.conf <= 0.0 {
                if line.text.trim().eq("") {
                    continue;
                }
                line.text = line.text.trim().to_string();
                texts.push(line.clone());
                line = Default::default();
                continue;
            }
            if line.text.trim().eq("") {
                line.x = dt.left;
                line.y = dt.top;
            }
            if line.height < dt.height {
                line.height = dt.height;
            }
            line.width += dt.width;
            line.text.push_str(&format!("{} ", dt.text));
        }
        Ok(texts)
    }

    pub fn ocr_image(&self, path: &str) -> Result<String, anyhow::Error> {
        let default_args = rusty_tesseract::Args {
            lang: self.code.to_owned(),
            ..rusty_tesseract::Args::default()
        };
        let image = Image::from_path(path)?;
        let text = rusty_tesseract::image_to_string(&image, &default_args)?;
        Ok(text)
    }
}
