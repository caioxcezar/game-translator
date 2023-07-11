mod imp;
use crate::rect::Rect;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::Settings;
use glib::{ clone, Object };
use gtk::{ gdk, gio, glib::{ self, MainContext }, Expression, PropertyExpression };
use headless_chrome::Browser;
use rusty_tesseract::{ Args, Image };
use std::str::FromStr;
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

        let action_new_profile = gio::SimpleAction::new("translate-image", None);
        action_new_profile.connect_activate(
            clone!(@weak self as window => move |_, _| {
            window.navigate("image")
        })
        );
        self.add_action(&action_new_profile);

        let action_new_profile = gio::SimpleAction::new("search-image", None);
        action_new_profile.connect_activate(
            clone!(@weak self as window => move |_, _| {
            window.search_image()
        })
        );
        self.add_action(&action_new_profile);
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
        self.build_drawing_area();

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

    fn build_drawing_area(&self) {
        // self.imp()
        //     .drawing_area
        //     .set_draw_func(move |_, cr, _width, _height| {
        //         let rgba = gdk::RGBA::from_str("DimGray").unwrap();
        //         // GdkCairoContextExt::set_source_rgba(cr, &rgba);
        //         let ret = gtk::gdk::Rectangle::new(10, 10, 100, 100);
        //         GdkCairoContextExt::add_rectangle(cr, &ret);
        //         // cr.paint().expect("Invalid cairo surface state");
        //         cr.stroke();
        //     });

        let drag = gtk::GestureDrag::new();

        // drag.connect_drag_end(|gesture, end_x, end_y| {
        //     let (start_x, start_y) = gesture.start_point().unwrap();
        //     GdkCairoContextExt::add_rectangle(
        //         &window,
        //         &gtk::cairo::Rectangle::new(start_x, start_y, end_x - start_x, end_y - start_y),
        //     )
        // });

        drag.connect_drag_end(
            clone!(@weak self as window => move |gesture, width, height| {
                let (x, y) = gesture.start_point().unwrap();
                let new_rect = Rect {
                    height: height as i32, width: width as i32, x: x as i32, y: y as i32
                };
                let areas = window.imp().translation_areas.try_borrow_mut();
                if areas.is_err() { return; }
                let mut areas = areas.unwrap();
                areas.push(new_rect);
                let areas = areas.clone();
                window.imp()
                    .drawing_area
                    .set_draw_func(move |_, cr, _width, _height| {
                        areas.iter().for_each(|area| {
                            let ret = gtk::gdk::Rectangle::new(area.x, area.y, area.width, area.height);
                            let rgba = gdk::RGBA::from_str("DimGray").unwrap();
                            GdkCairoContextExt::set_source_rgba(cr, &rgba);
    
                            GdkCairoContextExt::add_rectangle(cr, &ret);
                            cr.stroke().expect("Invalid cairo surface state");
                        });
                    });
            })
        );

        self.imp().drawing_area.add_controller(drag);

        // let click = gtk::GestureClick::new();

        // click.connect_pressed(|_, _, _, _| {
        //     println!("pressed");
        // });

        // click.connect_released(|_, _, _, _| {
        //     println!("released");
        // });

        // let click = gtk::GestureDrag::new();

        // self.imp().drawing_area.add_controller(click);
    }

    fn ocr_image(&self, path: &str) {
        let mut default_args = Args::default();
        let image = Image::from_path(path);
        if let Ok(image) = image {
            let lang = self.imp().dd_ocr.selected_item().and_downcast::<OcrObject>().unwrap();
            default_args.lang = lang.code();
            let output = rusty_tesseract::image_to_string(&image, &default_args).unwrap();
            self.imp().ocr_frame.set_text(&output);

            let main_context = MainContext::default();

            main_context.spawn_local(
                clone!(@weak self as window => async move {
                let ocr = OcrData { code: lang.code(), language: lang.language() };
                window.translate(&ocr.to_translator().code, &output).await;
            })
            );
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

    async fn translate(&self, source: &str, text: &str) {
        let target = self
            .imp()
            .dd_translation.selected_item()
            .and_downcast::<TranslatorObject>()
            .unwrap();
        let translated_text = self.translate_from_deepl(
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
