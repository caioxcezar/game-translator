mod imp;
use crate::{ rect::Rect, window_manager::sys::WindowManager };
use crate::state::State;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::Settings;
use glib::{ clone, Object };
use gtk::{ gio, glib::{ self }, Expression, PropertyExpression };
use headless_chrome::Browser;
use rusty_tesseract::{ Args, Image };
use screenshots::Screen;
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

const WINDOW_NAME: &str = "GT Overlay";

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
        self.add_simple_action(
            "new-profile",
            clone!(@weak self as window => move |_, _| window.navigate("main"))
        );

        self.add_simple_action(
            "translate-image",
            clone!(@weak self as window => move |_, _| window.navigate("image"))
        );

        self.set_language_action();

        self.add_simple_action(
            "search-image",
            clone!(@weak self as window => move |_, _| window.search_image())
        );

        self.add_simple_action(
            "on-action",
            clone!(@weak self as window => move |_, _| window.on_action())
        );

        self.add_simple_action(
            "configure-page",
            clone!(@weak self as window => move |_, _| window.configure_page())
        );
    }

    fn add_simple_action<F: Fn(&gio::SimpleAction, std::option::Option<&glib::Variant>) + 'static>(
        &self,
        name: &str,
        callback: F
    ) {
        let action = gio::SimpleAction::new(name, None);
        action.connect_activate(callback);
        self.add_action(&action);
    }

    fn add_toggle_action<F: Fn(&gio::SimpleAction, std::option::Option<&glib::Variant>) + 'static>(
        &self,
        name: &str,
        value: &str,
        f: F
    ) {
        let action = gio::SimpleAction::new_stateful(
            name,
            Some(glib::VariantTy::STRING),
            glib::Variant::from(value)
        );
        action.connect_change_state(f);
        self.add_action(&action);
    }

    fn set_language_action(&self) {
        let provider = self.settings().string("tra-provider");
        self.add_toggle_action(
            "toggle-language",
            provider.as_str(),
            clone!(@weak self as window => move |action, value| {
                let new_value = value.unwrap().to_owned();
                let str_value = new_value.str().unwrap();
                let _ = window.settings().set("tra-provider", str_value);
                action.set_state(new_value);
            })
        )
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

    fn setup_drag_action(&self) {
        let controller = gtk::GestureDrag::new();
        controller.connect_drag_end(
            clone!(@weak self as window => move |gesture, width, height| {
                let (x, y) = gesture.start_point().unwrap();
                let mut x = x as i32;
                let mut y = y as i32;
                let mut height = height as i32;
                let mut width = width as i32;
                let areas = window.imp().translation_areas.try_borrow_mut();
                if areas.is_err() { return; }
                let mut areas = areas.unwrap();
                let mut can_add = true;

                if width == 0 && height == 0 {
                    can_add = false;
                    areas.retain_mut(|area| x < area.x || x > area.x + area.width || y < area.y || y > area.y + area.height);
                } else {
                    areas.iter_mut().for_each(|area| {
                        if x < area.x || x > area.x + area.width || y < area.y || y > area.y + area.height {
                            return;
                        }
                        area.x += width;
                        area.y += height;
                        can_add = false;
                    });
                }

                if height < 0 {
                    height = -height;
                    y -= height;
                }
                if width < 0 {
                    width = -width;
                    x -= width;
                }
                let new_rect = Rect { height, width, x, y, ..Default::default() };
                
                can_add = can_add && !areas.iter().any(|rect| {
                    let x_overlap = value_in_range(new_rect.x, rect.x, rect.x + rect.width) || value_in_range(rect.x, new_rect.x, new_rect.x + new_rect.width);
                    let y_overlap = value_in_range(new_rect.y, rect.y, rect.y + rect.height) || value_in_range(rect.y, new_rect.y, new_rect.y + new_rect.height);
                    x_overlap && y_overlap
                });

                if can_add { areas.push(new_rect); }

                let areas = areas.clone();
                window.draw_rectagles(areas);
            })
        );
        self.imp().drawing_area.add_controller(controller);
    }

    fn draw_rectagles(&self, areas: Vec<Rect>) {
        self.imp().drawing_area.set_draw_func(move |_, cr, _width, _height| {
            cr.set_source_rgba(250.0, 0.0, 250.0, 1.0);
            areas.iter().for_each(|area| {
                let ret = gtk::gdk::Rectangle::new(area.x, area.y, area.width, area.height);
                cr.add_rectangle(&ret);
            });
            cr.stroke().expect("Invalid cairo surface state");
        });
    }

    fn open_overlay_page(&self, intangible: bool) {
        WindowManager::close_window(WINDOW_NAME);
        let page = gtk::Window
            ::builder()
            .title(WINDOW_NAME)
            .name("translation-page")
            .maximized(true)
            .decorated(false)
            .child(&self.imp().drawing_area)
            .css_classes(["overlay"].to_vec())
            .build();
        page.set_visible(true);
        WindowManager::set_window_translucent(WINDOW_NAME, intangible);
    }

    fn on_action(&self) {
        let state = self.imp().state.borrow().clone();
        let state = match state {
            State::Stopped | State::Paused => self.start(),
            State::Started => self.stop(),
        };
        self.imp().state.replace(state);
    }

    fn start(&self) -> State {
        self.text_overlay(false);
        self.imp().action_button.set_label("Stop");
        State::Started
    }

    fn stop(&self) -> State {
        WindowManager::close_window(WINDOW_NAME);
        self.imp().action_button.set_label("Start");
        State::Stopped
    }

    fn configure_page(&self) {
        self.open_overlay_page(false);
        let areas = self.imp().translation_areas.try_borrow();
        if areas.is_err() {
            return;
        }
        self.draw_rectagles(areas.unwrap().clone());
        self.imp().action_button.set_label("Start");
        self.imp().state.replace(State::Paused);
    }

    fn text_overlay(&self, translate: bool) {
        self.open_overlay_page(true);
        let lang = self.imp().dd_ocr.selected_item().and_downcast::<OcrObject>().unwrap();
        let target = self
            .imp()
            .dd_translation.selected_item()
            .and_downcast::<TranslatorObject>()
            .unwrap();
        let _ = if self.imp().chk_full_screen.is_active() {
            self.ocr_screen(&lang)
        } else {
            self.ocr_areas(&lang)
        };
        if translate {
            let _ = self.translate_from_ocr(&lang, &target);
        }
        let _ = self.draw_text();
    }

    fn ocr_screen(&self, lang: &OcrObject) -> Result<(), anyhow::Error> {
        let default_args = rusty_tesseract::Args { lang: lang.code(), ..Default::default() };
        let screens = Screen::all()?;
        let screen = screens[0];
        let screenshot = screen.capture()?;
        screenshot.save("target/current_capture.png")?;
        let image = Image::from_path("target/current_capture.png")?;
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
                line.y = dt.top;
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
            cr.set_antialias(gtk::cairo::Antialias::Fast);
            for rect in rects.iter() {
                draw_line(cr, rect);
            }
            cr.stroke().expect("Invalid cairo surface state");
        });
        Ok(())
    }

    fn ocr_areas(&self, lang: &OcrObject) -> Result<(), anyhow::Error> {
        let screens = Screen::all()?;
        let screen = screens[0];
        let areas = self.imp().translation_areas.try_borrow()?;
        let areas = areas.clone();
        let default_args = rusty_tesseract::Args { lang: lang.code(), ..Default::default() };
        let rects = areas
            .par_iter()
            .flat_map(
                |area| -> Result<Rect, anyhow::Error> {
                    let result = ocr_area(area, &screen, &default_args);
                    if result.is_err() {
                        println!("Error: {:?}", &result);
                    }
                    result
                }
            )
            .collect::<Vec<Rect>>();
        self.imp().texts.replace(rects);
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

    fn translate_from_ocr(
        &self,
        lang: &OcrObject,
        target: &TranslatorObject
    ) -> Result<(), anyhow::Error> {
        let ocr = OcrData { code: lang.code(), language: lang.language() };
        let mut rects = self.imp().texts.try_borrow_mut()?;
        if rects.is_empty() {
            return Ok(());
        }
        let text = rects
            .iter()
            .map(|rect| rect.text.clone())
            .collect::<Vec<String>>()
            .join("\n=+=\n");
        let text = (match self.settings().string("tra-provider").as_str() {
            "deepl" =>
                self.translate_from_deepl(
                    &target.code(),
                    &ocr.to_translator().code,
                    &urlencoding::encode(&text)
                ),
            _ =>
                self.translate_from_google(
                    &target.code(),
                    &ocr.to_translator().code,
                    &urlencoding::encode(&text)
                ),
        })?;
        let texts = text.split("\n=+=\n").collect::<Vec<&str>>();
        for (i, tx) in texts.iter().enumerate() {
            rects[i].text = tx.to_string();
        }
        Ok(())
    }
}

fn ocr_area(area: &Rect, screen: &Screen, default_args: &Args) -> Result<Rect, anyhow::Error> {
    let path = format!("target/{}.png", Uuid::new_v4());
    let screenshot = screen.capture_area(area.x, area.y, area.width as u32, area.height as u32)?;
    screenshot.save(&path)?;
    let image = Image::from_path(&path)?;
    let text = rusty_tesseract::image_to_string(&image, default_args)?.trim().to_string();
    fs::remove_file(&path)?;
    Ok(Rect { text, ..area.clone() })
}

fn draw_line(cr: &gtk::cairo::Context, rect: &Rect) {
    let x = rect.x as f64;
    let y = rect.y as f64;
    cr.select_font_face(
        "times, serif",
        gtk::cairo::FontSlant::Normal,
        gtk::cairo::FontWeight::Normal
    );
    let chars: Vec<char> = rect.text.chars().collect();
    if chars.is_empty() {
        return;
    }
    let lines = rect.text.lines();
    let font_size = (rect.height / (lines.clone().count() as i32)) as f64;
    cr.set_font_size(font_size);
    let mut pos_y = font_size;
    for line in lines {
        draw_text_with_outline(cr, x, y + pos_y, line);
        pos_y += font_size;
    }
}

fn draw_text_with_outline(cr: &gtk::cairo::Context, x: f64, y: f64, text: &str) {
    cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
    for _x in [x - 1.5, x + 1.5] {
        for _y in [y - 1.5, y + 1.5] {
            cr.move_to(_x, _y);
            let _ = cr.show_text(text);
        }
    }
    cr.move_to(x, y);
    cr.set_source_rgba(255.0, 255.0, 255.0, 1.0);
    let _ = cr.show_text(text);
}

fn value_in_range(value: i32, min: i32, max: i32) -> bool {
    value >= min && value <= max
}
