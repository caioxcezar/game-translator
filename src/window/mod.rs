mod imp;
use crate::rect::Rect;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::Settings;
use glib::{ clone, Object };
use gtk::{ gio, glib::{ self }, Expression, PropertyExpression };
use headless_chrome::Browser;
use rusty_tesseract::{ Args, Image };
use screenshots::{ Screen, Compression };
use std::fs;
use crate::{
    ocr_object::{ OcrData, OcrObject },
    translator_object::{ TranslatorData, TranslatorObject },
    APP_ID,
};

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn setup_settings(&self) {
        let settings = Settings::new(APP_ID);
        self.imp()
            .settings.set(settings)
            .expect("`settings` should not be set before calling `setup_settings`.");
    }

    fn settings(&self) -> &Settings {
        self.imp().settings.get().expect("`settings` should be set in `setup_settings`.")
    }

    fn setup_actions(&self) {
        let action_new_profile = gio::SimpleAction::new("new-profile", None);
        action_new_profile.connect_activate(
            clone!(@weak self as window => move |_, _| {
            window.navigate("main")
        })
        );
        self.add_action(&action_new_profile);

        let translate_image = gio::SimpleAction::new("translate-image", None);
        translate_image.connect_activate(
            clone!(@weak self as window => move |_, _| {
            window.navigate("image")
        })
        );
        self.add_action(&translate_image);

        let search_image = gio::SimpleAction::new("search-image", None);
        search_image.connect_activate(
            clone!(@weak self as window => move |_, _| {
            // window.search_image()
            window.translation_page()
        })
        );
        self.add_action(&search_image);
    }

    fn navigate(&self, page: &str) {
        self.imp().stack.set_visible_child_name(page);
    }

    fn setup_data(&self) {
        let ocr_lang = self.settings().uint("ocr-lang");
        let tra_lang = self.settings().uint("tra-lang");

        self.imp().translation_areas.replace(Vec::new());
        let languages = rusty_tesseract::get_tesseract_langs();
        match languages {
            Ok(values) => {
                let list = gio::ListStore::new(OcrObject::static_type());
                for lang in values {
                    list.append(&OcrObject::new(lang));
                }
                let expression = PropertyExpression::new(
                    OcrObject::static_type(),
                    Expression::NONE,
                    "language"
                );
                self.imp().dd_ocr.set_expression(Some(expression));
                self.imp().dd_ocr.set_model(Some(&list));
                self.imp().dd_ocr.set_selected(ocr_lang);
            }
            Err(value) =>
                self.dialog(
                    "Can't find languages for translation",
                    &format!("{}\r\nPossible cause of the problem: Tesseract is not installed in your system. Please follow the instructions at https://tesseract-ocr.github.io/tessdoc/Installation.html", value)
                ),
        }
        let list = gio::ListStore::new(TranslatorObject::static_type());
        for lang in TranslatorData::all_languages() {
            list.append(&TranslatorObject::new(lang.code));
        }
        let expression = PropertyExpression::new(
            TranslatorObject::static_type(),
            Expression::NONE,
            "language"
        );
        self.imp().dd_translation.set_expression(Some(expression));
        self.imp().dd_translation.set_model(Some(&list));
        self.imp().dd_translation.set_selected(tra_lang);

        //self.navigate("main")
        self.navigate("image")
    }

    fn dialog(&self, message: &str, detail: &str) {
        let dialog = gtk::AlertDialog
            ::builder()
            .detail(detail)
            .message(message)
            .modal(true)
            .build();
        dialog.show(Some(self))
    }

    fn translation_page(&self) {
        self.ocr_screen();

        let page = gtk::Window
            ::builder()
            .maximized(true)
            .decorated(false)
            .child(&self.imp().drawing_area)
            .css_classes(["overlay"].to_vec())
            .build();

        page.set_visible(true);
    }

    fn search_image(&self) {
        let dd_ocr = self.imp().dd_ocr.selected();
        let dd_translation = self.imp().dd_translation.selected();
        let _ = self.settings().set("ocr-lang", dd_ocr);
        let _ = self.settings().set("tra-lang", dd_translation);

        let file_dialog = gtk::FileDialog::builder().title("Select a image to translate").build();

        file_dialog.open(
            Some(self),
            gio::Cancellable::NONE,
            clone!(@weak self as window => move |result| {
                if result.is_err() { return }
                let file = result.unwrap();
                let path = file.path().unwrap();
                window.ocr_image(path.to_str().unwrap());
                window.imp().picture.set_file(Some(&file))
            })
        );
    }

    fn draw_text(&self) {
        let areas = self.imp().translation_areas.try_borrow_mut();
        if areas.is_err() {
            return;
        }
        let areas = areas.unwrap();
        let areas = areas.clone();
        self.imp().drawing_area.set_draw_func(move |_, cr, _width, _height| {
            cr.set_source_rgba(250.0, 0.0, 250.0, 1.0);
            cr.set_antialias(gtk::cairo::Antialias::Fast);
            areas.iter().for_each(|area| {
                cr.move_to(area.x as f64, area.y as f64);
                cr.select_font_face(
                    "Noto Sans",
                    gtk::cairo::FontSlant::Normal,
                    gtk::cairo::FontWeight::Normal
                );
                cr.set_font_size(32.0);
                let _ = cr.show_text(&area.text);
            });
            cr.stroke().expect("Invalid cairo surface state");
        });
    }

    fn ocr_screen(&self) {
        let mut default_args = Args::default();
        let screens = Screen::all().unwrap();
        let screen = screens[0];
        let screenshot = screen.capture();
        if screenshot.is_err() {
            return;
        }
        let screenshot = screenshot.unwrap();
        let buffer = screenshot.to_png(Compression::Fast).unwrap();
        fs::write("target/current_capture.png", buffer).unwrap();
        let image = Image::from_path("target/current_capture.png");
        if let Ok(image) = image {
            let lang = self.imp().dd_ocr.selected_item().and_downcast::<OcrObject>().unwrap();
            default_args.lang = lang.code();
            let output = rusty_tesseract::image_to_data(&image, &default_args).unwrap();
            let mut areas = Vec::new();
            for dt in output.data {
                if dt.conf <= 0.0 {
                    continue;
                }
                let new_rect = Rect {
                    height: dt.height,
                    width: dt.width,
                    x: dt.left,
                    y: dt.top,
                    text: dt.text,
                };
                areas.push(new_rect);
            }
            self.imp().translation_areas.replace(areas);
            self.draw_text();
        }
    }

    fn ocr_image(&self, path: &str) {
        let mut default_args = Args::default();
        let image = Image::from_path(path);
        if let Ok(image) = image {
            let lang = self.imp().dd_ocr.selected_item().and_downcast::<OcrObject>().unwrap();
            default_args.lang = lang.code();
            let ocr = OcrData { code: lang.code(), language: lang.language() };
            let text = rusty_tesseract::image_to_string(&image, &default_args).unwrap();

            self.imp().ocr_frame.set_text(&text);
            self.translate(&ocr.to_translator().code, &text);
        }
    }

    fn translate_from_deepl(
        &self,
        target: &str,
        source: &str,
        text: &str
    ) -> Result<String, anyhow::Error> {
        let path = "//span[contains(@class, \"sentence_highlight\")]";
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

    fn translate_from_google(
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

    fn translate(&self, source: &str, text: &str) {
        let target = self
            .imp()
            .dd_translation.selected_item()
            .and_downcast::<TranslatorObject>()
            .unwrap();
        let translated_text = self.translate_from_google(
            &target.code(),
            source,
            &urlencoding::encode(text)
        );
        match translated_text {
            Ok(txt) => { self.imp().translator_frame.set_text(&txt) }
            Err(err) => { self.dialog("Translation error", &err.to_string()) }
        }
    }
}
