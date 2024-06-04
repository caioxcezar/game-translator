mod imp;
use crate::screen_object::{ ScreenData, ScreenObject };
use crate::{ rect::Rect, window_manager::sys::WindowManager };
use crate::state::State;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::Settings;
use glib::{ clone, Object };
use gtk::{ gio, glib::{ self }, Expression, PropertyExpression };
use crate::{
    ocr_object::{ OcrData, OcrObject },
    translator_object::{ TranslatorData, TranslatorObject },
    APP_ID,
};
use std::thread;

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

    fn current_state(&self) -> State {
        self.imp().state.borrow().clone()
    }

    fn ocr_data(&self) -> Result<OcrData, anyhow::Error> {
        let lang = self.imp().dd_ocr.selected_item().and_downcast::<OcrObject>();
        if let Some(lang) = lang {
            return Ok(OcrData {
                code: lang.code(),
                language: lang.language(),
            });
        }
        Err(anyhow::anyhow!("No OCR language selected"))
    }

    fn screen_data(&self) -> Result<ScreenData, anyhow::Error> {
        let screen = self.imp().dd_screen.selected_item().and_downcast::<ScreenObject>();
        if let Some(screen) = screen {
            return Ok(ScreenData {
                id: screen.id(),
                app_name: screen.app_name(),
                title: screen.title(),
            });
        }
        Err(anyhow::anyhow!("No screen selected"))
    }

    fn translator_data(&self) -> Result<TranslatorData, anyhow::Error> {
        let lang = self.imp().dd_translation.selected_item().and_downcast::<TranslatorObject>();
        if let Some(lang) = lang {
            return Ok(TranslatorData { code: lang.code(), language: lang.language() });
        }
        Err(anyhow::anyhow!("No translation language selected"))
    }

    fn translation_areas(&self) -> Result<Vec<Rect>, anyhow::Error> {
        Ok(self.imp().translation_areas.try_borrow()?.clone())
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

        let list = gio::ListStore::new(ScreenObject::static_type());
        if let Ok(windows) = xcap::Window::all() {
            for win in windows {
                let title = win.title().to_string();
                if title.is_empty() {
                    continue;
                }
                list.append(&ScreenObject::new(win.id(), win.app_name().to_string(), title));
            }
        }

        let expression = PropertyExpression::new(
            ScreenObject::static_type(),
            Expression::NONE,
            "title"
        );

        self.imp().dd_screen.set_expression(Some(expression));
        self.imp().dd_screen.set_model(Some(&list));
        self.imp().dd_screen.set_selected(0);

        self.navigate("main");
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
        let state = match self.current_state() {
            State::Stopped | State::Paused => self.start(),
            State::Started => self.stop(),
        };
        self.imp().state.replace(state);
    }

    fn start(&self) -> State {
        self.open_overlay_page(true);
        self.imp().action_button.set_label("Stop");
        if let Err(err) = self.text_overlay(true) {
            self.dialog("Text Overlay Error", &err.to_string());
        }
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

    fn text_overlay(&self, translate: bool) -> Result<(), anyhow::Error> {
        let ocr = self.ocr_data()?;
        let screen = self.screen_data()?;
        let translator = self.translator_data()?;
        let provider = self.settings().string("tra-provider");
        let areas = self.translation_areas()?;
        let is_areas = !self.imp().chk_full_screen.is_active();

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        thread::spawn(move || {
            let _ = tx.send(
                (|| -> Result<Vec<Rect>, anyhow::Error> {
                    let texts = if is_areas {
                        ocr.ocr_areas(&areas, &screen)
                    } else {
                        ocr.ocr_screen(&screen)
                    };
                    if translate {
                        translator.translate_from_ocr(&ocr, provider.as_str(), texts?)
                    } else {
                        texts
                    }
                })()
            );
        });

        rx.attach(
            None,
            clone!(@weak self as window => @default-return glib::Continue(false), move |result| {
                if let Ok(texts) = result { let _ = window.draw_text(texts); }
                if window.current_state() == State::Started { let _ = window.text_overlay(true); }
                glib::Continue(false)
            })
        );

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
                let _ = window.ocr_from_img(&file);
            })
        );
    }

    fn ocr_from_img(&self, file: &gio::File) -> Result<(), anyhow::Error> {
        let path = file.path();

        if file.path().is_none() {
            return Err(anyhow::anyhow!("No image selected"));
        }

        let path = path.unwrap();
        let ocr = self.ocr_data()?;
        let translator = self.translator_data()?;
        let text = ocr.ocr_image(path.to_str().unwrap())?;
        let provider = self.settings().string("tra-provider");

        self.imp().ocr_frame.set_text(&text);
        let translated_text = translator.translate(
            &ocr.to_translator().code,
            provider.as_str(),
            &text
        );

        match translated_text {
            Ok(txt) => {
                self.imp().translator_frame.set_text(&txt);
            }
            Err(err) => {
                self.dialog("Translation error", &err.to_string());
            }
        }

        self.imp().picture.set_file(Some(file));

        Ok(())
    }

    fn draw_text(&self, texts: Vec<Rect>) -> Result<(), anyhow::Error> {
        self.imp().drawing_area.queue_draw();
        self.imp().drawing_area.set_draw_func(move |_, cr, _width, _height| {
            cr.set_antialias(gtk::cairo::Antialias::Fast);
            for text in texts.iter() {
                draw_line(cr, text);
            }
            cr.stroke().expect("Invalid cairo surface state");
        });
        Ok(())
    }
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

fn _clean(drawing_area: &gtk::DrawingArea) -> Result<(), anyhow::Error> {
    drawing_area.set_draw_func(move |_, cr, _width, _height| {
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        cr.set_operator(gtk::cairo::Operator::Clear);
        cr.rectangle(0.0, 0.0, _width as f64, _height as f64);
        let _ = cr.paint_with_alpha(1.0);
    });
    Ok(())
}
