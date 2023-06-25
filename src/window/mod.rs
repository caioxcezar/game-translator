mod imp;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::Settings;
use glib::{clone, Object};
use gtk::{
    gio,
    glib::{self, MainContext},
    Expression, PropertyExpression,
};
use libretranslate::{translate, Language};
use rusty_tesseract::{Args, Image};
use std::str::FromStr;

use crate::{
    ocr_object::{OcrData, OcrObject},
    translator_object::{TranslatorData, TranslatorObject},
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
            .settings
            .set(settings)
            .expect("`settings` should not be set before calling `setup_settings`.");
    }

    fn settings(&self) -> &Settings {
        self.imp()
            .settings
            .get()
            .expect("`settings` should be set in `setup_settings`.")
    }

    fn setup_actions(&self) {
        // let save_action = self.settings().create_action("save");
        // self.add_action(&save_action);

        // let import_action = self.settings().create_action("import");
        // self.add_action(&import_action);
        let action_new_profile = gio::SimpleAction::new("new-profile", None);
        action_new_profile.connect_activate(clone!(@weak self as window => move |_, _| {
            window.navigate("main")
        }));
        self.add_action(&action_new_profile);

        let action_new_profile = gio::SimpleAction::new("translate-image", None);
        action_new_profile.connect_activate(clone!(@weak self as window => move |_, _| {
            window.navigate("image")
        }));
        self.add_action(&action_new_profile);

        let action_new_profile = gio::SimpleAction::new("search-image", None);
        action_new_profile.connect_activate(clone!(@weak self as window => move |_, _| {
            window.search_image()
        }));
        self.add_action(&action_new_profile);
    }

    fn navigate(&self, page: &str) {
        self.imp().stack.set_visible_child_name(page);
    }

    fn setup_data(&self) {
        let languages = rusty_tesseract::get_tesseract_langs();
        match languages {
            Ok(values) => {
                let list = gio::ListStore::new(OcrObject::static_type());
                for lang in values {
                    list.append(&OcrObject::new(lang));
                }
                let expression = PropertyExpression::new(OcrObject::static_type(), Expression::NONE, "language");
                self.imp().drop_down_ocr.set_expression(Some(expression));
                self.imp().drop_down_ocr.set_model(Some(&list));
            }
            Err(value) => self.dialog("Can't find languages for translation", &format!("{}\r\nPossible cause of the problem: Tesseract is not installed in your system. Please follow the instructions at https://tesseract-ocr.github.io/tessdoc/Installation.html", value)),
        }
        let list = gio::ListStore::new(TranslatorObject::static_type());
        for lang in TranslatorData::all_languages() {
            list.append(&TranslatorObject::new(lang.code));
        }
        let expression = PropertyExpression::new(
            TranslatorObject::static_type(),
            Expression::NONE,
            "language",
        );
        self.imp()
            .drop_down_translation
            .set_expression(Some(expression));
        self.imp().drop_down_translation.set_model(Some(&list));
        self.navigate("main")
    }

    fn dialog(&self, message: &str, detail: &str) {
        let dialog = gtk::AlertDialog::builder()
            .detail(detail)
            .message(message)
            .modal(true)
            .build();
        dialog.show(Some(self))
    }

    fn search_image(&self) {
        let file_dialog = gtk::FileDialog::builder()
            .title("Select a image to translate")
            .build();

        file_dialog.open(
            Some(self),
            gio::Cancellable::NONE,
            clone!(@weak self as window => move |result| {
                if result.is_err() { return }
                let file = result.unwrap();
                let path = file.path().unwrap();
                window.ocr_image(path.to_str().unwrap());
                window.imp().picture.set_file(Some(&file))
            }),
        );
    }

    fn ocr_image(&self, path: &str) {
        let mut default_args = Args::default();
        let image = Image::from_path(path);
        if let Ok(image) = image {
            let lang = self
                .imp()
                .drop_down_ocr
                .selected_item()
                .and_downcast::<OcrObject>()
                .unwrap();
            default_args.lang = lang.code();
            let output = rusty_tesseract::image_to_string(&image, &default_args).unwrap();
            self.imp().ocr_frame.set_text(&output);

            let main_context = MainContext::default();

            main_context.spawn_local(clone!(@weak self as window => async move {
                let ocr = OcrData { code: lang.code(), language: lang.language() };
                window.translate(&ocr.to_translator().code, &output).await;
            }));
        }
    }

    async fn translate(&self, source: &str, text: &str) {
        let target = self
            .imp()
            .drop_down_translation
            .selected_item()
            .and_downcast::<TranslatorObject>()
            .unwrap();

        let s = Language::from_str(source).unwrap();
        let t = Language::from_str(&target.code()).unwrap();
        let data = translate(s, t, text, None).await;
        match data {
            Ok(value) => {
                self.imp().ocr_frame.set_text(&value.output);
            }
            Err(value) => self.dialog("Can't translate", &value.to_string()),
        }
    }
}
