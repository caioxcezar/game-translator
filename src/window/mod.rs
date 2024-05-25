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
use uuid::Uuid;
use rayon::prelude::*;

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
            window.search_image()
        })
        );
        self.add_action(&search_image);

        let translate_page = gio::SimpleAction::new("translate-page", None);
        translate_page.connect_activate(
            clone!(@weak self as window => move |_, _| {
            window.translation_page()
        })
        );
        self.add_action(&translate_page);
    }

    fn navigate(&self, page: &str) {
        self.imp().stack.set_visible_child_name(page);
    }

    fn setup_data(&self) {
        let ocr_lang = self.settings().uint("ocr-lang");
        let tra_lang = self.settings().uint("tra-lang");

        self.imp().texts.replace(Vec::new());
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

        self.navigate("main")
        //self.navigate("image")
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

    fn set_drag_action(&self) {
        let controller = gtk::GestureDrag::new();
        controller.connect_drag_end(
            clone!(@weak self as window => move |gesture, width, height| {
                let (x, y) = gesture.start_point().unwrap();
                let new_rect = Rect {
                    height: height as i32, width: width as i32, x: x as i32, y: y as i32, ..Default::default()
                };
                let areas = window.imp().translation_areas.try_borrow_mut();
                if areas.is_err() { return; }
                let mut areas = areas.unwrap();
                areas.push(new_rect);
                let areas = areas.clone();
                window.imp()
                    .drawing_area
                    .set_draw_func(move |_, cr, _width, _height| {
                        cr.set_source_rgba(250.0, 0.0, 250.0, 1.0);
                        areas.iter().for_each(|area| {
                            let ret = gtk::gdk::Rectangle::new(area.x, area.y, area.width, area.height);
                            cr.add_rectangle(&ret);
                        });
                        cr.stroke().expect("Invalid cairo surface state");
                    });
            })
        );
        self.imp().drawing_area.add_controller(controller);
    }

    fn translation_page(&self) {
        let _ = self.ocr_screen();
        // let _ = self.translate_from_ocr();
        //let _ = self.draw_text();
        let page = gtk::Window
            ::builder()
            .maximized(true)
            .decorated(true)
            .child(&self.imp().drawing_area)
            .css_classes(["overlay"].to_vec())
            .build();
        page.set_visible(true);
        self.set_drag_action();
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

    fn draw_text(&self) -> Result<(), anyhow::Error> {
        let rects = self.imp().texts.try_borrow_mut()?;
        let rects = rects.clone();
        self.imp().drawing_area.set_draw_func(move |_, cr, _width, _height| {
            cr.set_source_rgba(250.0, 0.0, 250.0, 1.0);
            cr.set_antialias(gtk::cairo::Antialias::Fast);
            for rect in rects.iter() {
                cr.move_to(rect.x as f64, rect.y as f64);
                cr.select_font_face(
                    "Noto Sans",
                    gtk::cairo::FontSlant::Normal,
                    gtk::cairo::FontWeight::Normal
                );
                let chars: Vec<char> = rect.text.chars().collect();
                if chars.is_empty() {
                    continue;
                }
                cr.set_font_size(rect.height as f64);
                let _ = cr.show_text(&rect.text);
            }
            cr.stroke().expect("Invalid cairo surface state");
        });
        Ok(())
    }

    fn ocr_area(&self) -> Result<(), anyhow::Error> {
        let screens = Screen::all()?;
        let screen = screens[0];
        let areas = self.imp().translation_areas.try_borrow()?;
        let areas = areas.clone();
        areas.par_iter().for_each(|area| {
            let id = Uuid::new_v4().to_string();
            let screenshot = screen
                .capture_area(area.x, area.y, area.width as u32, area.height as u32)
                .unwrap();
            let buffer = screenshot.to_png(Compression::Fast).unwrap();
            fs::write(format!("target/{}.png", id), buffer).unwrap();
        });
        Ok(())
    }

    fn ocr_screen(&self) -> Result<(), anyhow::Error> {
        let mut default_args = Args::default();
        let screens = Screen::all()?;
        let screen = screens[0];
        let screenshot = screen.capture()?;
        let buffer = screenshot.to_png(Compression::Fast)?;
        fs::write("target/current_capture.png", buffer)?;
        let image = Image::from_path("target/current_capture.png")?;
        let lang = self.imp().dd_ocr.selected_item().and_downcast::<OcrObject>().unwrap();
        default_args.lang = lang.code();
        let output = rusty_tesseract::image_to_data(&image, &default_args)?;
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
                line.y = dt.top + dt.height;
            }
            if line.height < dt.height {
                line.height = dt.height;
            }
            line.width += dt.width;
            line.text.push_str(&format!("{} ", dt.text));
        }
        self.imp().texts.replace(texts);
        Ok(())
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
        let provider = self.settings().string("tra-provider");
        let translated_text = match provider.as_str() {
            "google" =>
                self.translate_from_google(&target.code(), source, &urlencoding::encode(text)),
            _ => self.translate_from_deepl(&target.code(), source, &urlencoding::encode(text)),
        };
        match translated_text {
            Ok(txt) => { self.imp().translator_frame.set_text(&txt) }
            Err(err) => { self.dialog("Translation error", &err.to_string()) }
        }
    }

    fn translate_from_ocr(&self) -> Result<(), anyhow::Error> {
        let lang = self.imp().dd_ocr.selected_item().and_downcast::<OcrObject>().unwrap();
        let ocr = OcrData { code: lang.code(), language: lang.language() };
        let mut rects = self.imp().texts.try_borrow_mut()?;
        let text = rects
            .iter()
            .map(|area| { area.text.clone() })
            .collect::<Vec<String>>()
            .join("\n");
        let target = self
            .imp()
            .dd_translation.selected_item()
            .and_downcast::<TranslatorObject>()
            .unwrap();
        let text = match self.settings().string("tra-provider").as_str() {
            "deepl" =>
                self.translate_from_deepl(
                    &target.code(),
                    &ocr.to_translator().code,
                    &urlencoding::encode(&text)
                )?,
            _ =>
                self.translate_from_google(
                    &target.code(),
                    &ocr.to_translator().code,
                    &urlencoding::encode(&text)
                )?,
        };
        let text = text.split('\n').collect::<Vec<&str>>();
        for (i, tx) in text.iter().enumerate() {
            rects[i].text = tx.to_string();
        }
        Ok(())
    }
}
