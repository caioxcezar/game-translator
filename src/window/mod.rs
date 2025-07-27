mod imp;

use crate::{
    area_object::{AreaData, AreaObject},
    ocr_object::{OcrData, OcrObject},
    profile_object::{ProfileData, ProfileObject},
    screen_object::{ScreenData, ScreenObject},
    settings::Settings,
    state::State,
    translation,
    translator_object::{TranslatorData, TranslatorObject},
    utils,
    window_manager::sys::WindowManager,
};
use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::{Context, Result};
use gio::{ListStore, SimpleAction};
use glib::{clone, Object};
use gtk::{gio, glib, pango, Expression, PropertyExpression};
use regex::Regex;
use std::{cell::RefMut, thread};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

const WINDOW_NAME: &str = "GT Overlay";
const PORT: u32 = 50682;
const TRANSLATION_DELAY: u64 = 10000;

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn setup_webdriver(&self) {
        glib::spawn_future_local(clone!(@weak self as window => async move {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if !translation::webdriver_path().unwrap().exists() {
                    let dialog = gtk::Window::builder()
                        .title("Installing webdriver...")
                        .modal(true)
                        .resizable(false)
                        .decorated(true)
                        .default_width(300)
                        .default_height(150)
                        .build();

                    let label = gtk::Label::new(Some("This message will close altomatic when finished"));
                    dialog.set_child(Some(&label));
                    dialog.present();

                    if let Err(err) = translation::download_webdriver().await {
                        window.dialog("Error downloading webdriver", &err.to_string());
                        return;
                    }
                    dialog.close();
                    window.dialog("Webdriver Installed", "Webdriver installed and program ready to be used");
                }
            });
            if let Err(err) = translation::run_webdriver(PORT) {
                window.dialog("error running webdriver", &err.to_string());
            }
        }));
    }

    fn setup_settings(&self) {
        let file = utils::open_file(utils::settings_path().expect("Failed to get settings path"));
        let settings = match file {
            Ok(file) => serde_json::from_reader(file).expect("Failed to read settings file"),
            Err(_) => Settings {
                ..Default::default()
            },
        };
        self.imp().settings.replace(settings);
    }

    fn settings(&self) -> RefMut<Settings> {
        self.imp().settings.borrow_mut()
    }

    fn current_state(&self) -> State {
        self.imp().state.borrow().clone()
    }

    fn ocr_data(&self) -> Result<OcrData> {
        let lang = self
            .imp()
            .dd_ocr
            .selected_item()
            .and_downcast::<OcrObject>();
        if let Some(lang) = lang {
            return Ok(OcrData {
                code: lang.code(),
                language: lang.language(),
                is_vertical: lang.is_vertical(),
            });
        }
        Err(anyhow::anyhow!("No OCR language selected"))
    }

    fn screen_data(&self) -> Result<ScreenData> {
        let screen = self
            .imp()
            .dd_screen
            .selected_item()
            .and_downcast::<ScreenObject>()
            .expect("No screen selected");
        Ok(ScreenData {
            id: screen.id(),
            app_name: screen.app_name(),
            title: screen.title(),
        })
    }

    fn translator_data(&self) -> Result<TranslatorData> {
        let lang = self
            .imp()
            .dd_translation
            .selected_item()
            .and_downcast::<TranslatorObject>();
        if let Some(lang) = lang {
            return Ok(TranslatorData {
                code: lang.code(),
                language: lang.language(),
            });
        }
        Err(anyhow::anyhow!("No translation language selected"))
    }

    fn translation_areas(&self) -> Result<Vec<AreaData>> {
        let areas = self
            .selected_profile()?
            .areas()
            .iter::<AreaObject>()
            .filter_map(Result::ok)
            .map(|area_object| area_object.area_data())
            .collect();
        Ok(areas)
    }

    fn setup_actions(&self) {
        let obj = self.imp();

        obj.title
            .connect_changed(clone!(@weak self as window => move |entry| {
                if let Ok(profile) = window.selected_profile() {
                    profile.set_title(entry.text().to_string());
                }
            }));

        obj.chk_full_screen
            .connect_toggled(clone!(@weak self as window => move |button| {
                window.imp().config_button.set_sensitive(!button.is_active());

                if let Ok(profile) = window.selected_profile() {
                    profile.set_use_areas(button.is_active());
                }
            }));

        obj.dd_translation.connect_selected_item_notify(
            clone!(@weak self as window => move |drop_down| {
                let tra_obj = drop_down.selected_item().and_downcast::<TranslatorObject>();
                if let Some(tra_obj) = tra_obj {
                    let _ = window.settings().set("tra-lang", tra_obj.code());

                    if let Ok(profile) = window.selected_profile() {
                        profile.set_translation(tra_obj.code());
                    }
                };
            }),
        );

        obj.dd_ocr
            .connect_selected_item_notify(clone!(@weak self as window => move |drop_down| {
                let ocr_obj = drop_down.selected_item().and_downcast::<OcrObject>();
                if let Some(ocr_obj) = ocr_obj {
                    let _ = window.settings().set("ocr-lang", ocr_obj.code());

                    if let Ok(profile) = window.selected_profile() {
                        profile.set_language(ocr_obj.code());
                    }
                };
            }));

        obj.dd_screen.connect_selected_item_notify(
            clone!(@weak self as window => move |drop_down| {
                    if let Some(app) = drop_down.selected_item().and_downcast::<ScreenObject>() {
                        if let Ok(profile) = window.selected_profile() {
                            profile.set_app_name(app.app_name());
                            profile.set_app_title(app.title());
                        }
                    }
            }),
        );

        self.add_simple_action(
            "new-profile",
            clone!(@weak self as window => move |_, _| {
                if let Err(err) = window.new_profile() {
                    window.error_dialog(&err.to_string());
                }
            }),
        );

        self.add_simple_action(
            "translate-image",
            clone!(@weak self as window => move |_, _| window.navigate("image")),
        );

        self.set_language_action();

        self.add_simple_action(
            "on-action",
            clone!(@weak self as window => move |_, _| window.on_action()),
        );

        self.add_simple_action(
            "configure-page",
            clone!(@weak self as window => move |_, _| window.configure_page()),
        );

        self.add_simple_action(
            "remove-profile",
            clone!(@weak self as window => move |_, _| window.remove_current_profile()),
        );

        self.add_simple_action(
            "refresh-windows",
            clone!(@weak self as window => move |_, _| {
                if let Err(err) = window.setup_dd_screen() {
                    window.dialog("Failed to load applications", &err.to_string());
                }
            }),
        );
    }

    fn add_simple_action<F: Fn(&SimpleAction, std::option::Option<&glib::Variant>) + 'static>(
        &self,
        name: &str,
        callback: F,
    ) {
        let action = SimpleAction::new(name, None);
        action.connect_activate(callback);
        self.add_action(&action);
    }

    fn add_toggle_action<F: Fn(&SimpleAction, std::option::Option<&glib::Variant>) + 'static>(
        &self,
        name: &str,
        value: &str,
        f: F,
    ) {
        let action = SimpleAction::new_stateful(
            name,
            Some(glib::VariantTy::STRING),
            &glib::Variant::from(value),
        );
        action.connect_change_state(f);
        self.add_action(&action);
    }

    fn set_language_action(&self) {
        let settings = self.settings();
        let provider = settings.tra_provider();
        self.add_toggle_action(
            "toggle-language",
            provider,
            clone!(@weak self as window => move |action, value| {
                let new_value = value.unwrap().to_owned();
                let str_value = new_value.str().unwrap();
                let _ = window.settings().set("tra-provider", str_value.to_string());
                action.set_state(&new_value);
            }),
        )
    }

    fn navigate(&self, page: &str) {
        self.imp().stack.set_visible_child_name(page);
    }

    fn setup_data(&self) {
        self.setup_dd_ocr();
        self.setup_dd_translation();
        let _ = self.setup_dd_screen();

        self.navigate("main");
    }

    fn setup_dd_ocr(&self) {
        let languages = rusty_tesseract::get_tesseract_langs();
        match languages {
            Ok(values) => {
                let list = ListStore::new::<OcrObject>();
                for lang in &values {
                    list.append(&OcrObject::new(lang.to_string()));
                }
                let expression = PropertyExpression::new(
                    OcrObject::static_type(),
                    Expression::NONE,
                    "language"
                );
                self.imp().dd_ocr.set_expression(Some(expression));
                self.imp().dd_ocr.set_model(Some(&list));
            }
            Err(value) =>
                self.dialog(
                    "Can't find languages for translation",
                    &format!("{value}\r\nPossible cause of the problem: Tesseract is not installed in your system. Please follow the instructions at https://tesseract-ocr.github.io/tessdoc/Installation.html")
                ),
        }
    }

    fn setup_dd_translation(&self) {
        let list = ListStore::new::<TranslatorObject>();
        let all_langs = TranslatorData::all_languages();
        for lang in &all_langs {
            list.append(&TranslatorObject::new(lang.code.to_string()));
        }
        let expression = PropertyExpression::new(
            TranslatorObject::static_type(),
            Expression::NONE,
            "language",
        );
        self.imp().dd_translation.set_expression(Some(expression));
        self.imp().dd_translation.set_model(Some(&list));
    }

    fn setup_dd_screen(&self) -> Result<()> {
        let list = open_windows()?;

        let expression =
            PropertyExpression::new(ScreenObject::static_type(), Expression::NONE, "title");

        self.imp().dd_screen.set_expression(Some(expression));
        self.imp().dd_screen.set_model(Some(&list));

        Ok(())
    }

    fn dialog(&self, message: &str, detail: &str) {
        let dialog = gtk::AlertDialog::builder()
            .detail(detail)
            .message(message)
            .modal(true)
            .build();
        dialog.show(Some(self))
    }

    // region: Profiles
    fn profiles(&self) -> &ListStore {
        self.imp()
            .profiles
            .get()
            .expect("`profiles` should be set in `setup_profiles`.")
    }

    fn selected_profile(&self) -> Result<ProfileObject> {
        let id = self.current_profile_index();
        let profile_data = self.profile(id)?;
        Ok(profile_data)
    }

    fn setup_profiles(&self) {
        let _ = self.imp().profiles.set(ListStore::new::<ProfileObject>());

        self.imp().profiles_list.bind_model(
            self.imp().profiles.get(),
            clone!(@weak self as window => @default-panic, move |obj| {
                let collection_object = obj
                    .downcast_ref::<ProfileObject>()
                    .expect("The object should be of type `ProfileObject`.");
                let row = window.create_collection_row(collection_object);
                row.upcast()
            }),
        );

        self.imp().profiles_list.connect_row_selected(
            clone!(@weak self as window => move |_, row| {
                if row.is_none() { return; }
                let _ = (|| -> Result<()> {
                    let index = row.context("Failed to get selected row")?.index();
                    let profile = window.profile(index as u32)?.to_profile_data();
                    let obj = window.imp();

                    obj.title.set_text(&profile.title);

                    let list = TranslatorData::all_languages();
                    let id = list
                        .iter()
                        .position(|value| { value.code.eq(&profile.translation) })
                        .unwrap_or(0);
                    obj.dd_translation.set_selected(id as u32);

                    let list = rusty_tesseract::get_tesseract_langs()?;
                    let id = list
                        .iter()
                        .position(|value| { value.eq(&profile.language) })
                        .unwrap_or(0);
                    obj.dd_ocr.set_selected(id as u32);

                    obj.chk_full_screen.set_active(profile.use_areas);

                    let _ = window.setup_dd_screen();
                    let model = obj.dd_screen.model().expect("Failed to get model");
                    let mut not_found = true;
                    for i in 0..model.n_items() {
                        let item = model.item(i).expect("Failed to get item");
                        if item
                            .downcast::<ScreenObject>()
                            .expect("Failed to downcast item")
                            .app_name()
                            .eq(&profile.app_name)
                        {
                            obj.dd_screen.set_selected(i);
                            not_found = false;
                            break;
                        }
                    }

                    if not_found {
                        if let Some(model) = obj.dd_screen.model() {
                            let liststore = model.downcast_ref::<ListStore>().expect("Não é possível obter a lista de aplicativos");
                            let title = format!("{} (Closed)", utils::truncate_string(&profile.app_title, 61));
                            liststore.insert(0, &ScreenObject::new(u32::MAX, profile.app_name.to_string(), title));
                            obj.dd_screen.set_selected(0);
                        }
                    }

                    Ok(())
                })();
        })
        );
    }

    fn restore_data(&self) -> Result<()> {
        let profiles = match utils::open_file(utils::data_path()?) {
            Ok(file) => {
                let backup_data: Vec<ProfileData> = serde_json::from_reader(file)?;

                backup_data
                    .into_iter()
                    .map(ProfileObject::from_profile_data)
                    .collect::<Vec<ProfileObject>>()
            }
            Err(_) => vec![],
        };

        let list_store = self.profiles();
        list_store.extend_from_slice(&profiles);

        if profiles.is_empty() {
            self.new_profile()?;
        }

        let obj = self.imp();
        let row = obj.profiles_list.row_at_index(0);
        obj.profiles_list.select_row(row.as_ref());

        Ok(())
    }

    fn profile(&self, index: u32) -> Result<ProfileObject> {
        let profile = self
            .profiles()
            .item(index)
            .and_downcast::<ProfileObject>()
            .context("The object needs to be a `ProfileObject`.")?;
        Ok(profile)
    }

    fn current_profile_index(&self) -> u32 {
        match self.imp().profiles_list.selected_row() {
            Some(row) => row.index() as u32,
            None => 0,
        }
    }

    fn new_profile(&self) -> Result<()> {
        let profiles = self.profiles();
        let n_items = profiles.n_items();
        if n_items > 0 {
            let profile = profiles.item(n_items - 1).and_downcast::<ProfileObject>();
            if profile.unwrap().title() == "[New Profile]" {
                return Ok(());
            }
        }

        let settings = self.settings();
        let ocr_lang = settings.ocr_lang();
        let tra_lang = settings.tra_lang();

        self.profiles()
            .append(&ProfileObject::from_profile_data(ProfileData {
                title: "[New Profile]".to_string(),
                app_name: self.screen_data()?.app_name,
                app_title: self.screen_data()?.title,
                language: ocr_lang.to_string(),
                translation: tra_lang.to_string(),
                use_areas: self.imp().chk_full_screen.is_active(),
                areas: vec![],
            }));
        Ok(())
    }

    fn create_collection_row(&self, collection_object: &ProfileObject) -> gtk::ListBoxRow {
        let label = gtk::Label::builder()
            .ellipsize(pango::EllipsizeMode::End)
            .xalign(0.0)
            .build();

        collection_object
            .bind_property("title", &label, "label")
            .sync_create()
            .build();

        gtk::ListBoxRow::builder().child(&label).build()
    }

    fn remove_current_profile(&self) {
        let index = self.current_profile_index();
        let profiles = self.profiles();
        profiles.remove(index);
        if profiles.n_items() == 0 {
            if let Err(err) = self.new_profile() {
                self.dialog("Failed to create new profile", &err.to_string());
            }
        }
        let obj = self.imp();
        let row = obj.profiles_list.row_at_index(0);
        obj.profiles_list.select_row(row.as_ref());
    }
    // endregion: Profiles

    fn setup_drag_action(&self) {
        let controller = gtk::GestureDrag::new();
        controller.connect_drag_end(
            clone!(@weak self as window => move |gesture, width, height| {
                let (x, y) = gesture.start_point().unwrap();
                let mut x = x as i32;
                let mut y = y as i32;
                let mut height = height as i32;
                let mut width = width as i32;
                let areas = window.translation_areas();
                if areas.is_err() { return; }
                let mut areas = areas.unwrap();
                let mut can_add = true;

                if width == 0 && height == 0 {
                    can_add = false;
                    areas.retain_mut(|area| x < area.x || x > area.x + area.width || y < area.y || y > area.y + area.height);
                } else if width == 0 || height == 0 {
                    can_add = false;
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
                let new_rect = AreaData { height, width, x, y, ..Default::default() };

                can_add = can_add && !areas.iter().any(|rect| {
                    let x_overlap = utils::value_in_range(new_rect.x, rect.x, rect.x + rect.width) || utils::value_in_range(rect.x, new_rect.x, new_rect.x + new_rect.width);
                    let y_overlap = utils::value_in_range(new_rect.y, rect.y, rect.y + rect.height) || utils::value_in_range(rect.y, new_rect.y, new_rect.y + new_rect.height);
                    x_overlap && y_overlap
                });

                if can_add { areas.push(new_rect); }

                let rectagles = areas.clone();
                window.draw_rectagles(rectagles);

                if let Ok(profile) = window.selected_profile() {
                    profile.areas().remove_all();
                    for area in &areas {
                        profile.areas().append(&AreaObject::from_area_data(area.clone()));
                    }
                }
            })
        );
        self.imp().drawing_area.add_controller(controller);
    }

    fn draw_rectagles(&self, areas: Vec<AreaData>) {
        self.imp()
            .drawing_area
            .set_draw_func(move |_, cr, _width, _height| {
                cr.set_source_rgba(250.0, 0.0, 250.0, 1.0);
                areas.iter().for_each(|area| {
                    let ret = gtk::gdk::Rectangle::new(area.x, area.y, area.width, area.height);
                    cr.add_rectangle(&ret);
                });
                cr.stroke().expect("Invalid cairo surface state");
            });
    }

    fn open_overlay_page(&self, intangible: bool) {
        let page = gtk::Window::builder()
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
        self.change_state(state);
    }

    fn start(&self) -> State {
        if *self.imp().running.borrow() {
            self.dialog(
                "Still Running",
                "Please wait until the previous translation is finished.",
            );
            return self.current_state();
        }
        if let Err(err) = self.check_application() {
            self.dialog("Application not valid", &err.to_string());
            return self.current_state();
        }
        self.open_overlay_page(true);
        if let Err(err) = self.text_overlay() {
            self.dialog("Text Overlay Error", &err.to_string());
            State::Stopped
        } else {
            State::Started
        }
    }

    fn check_application(&self) -> Result<()> {
        let ocr = self.screen_data()?;

        let list = open_windows()?;
        for i in 0..list.n_items() {
            let item = list.item(i).expect("Failed to get item");
            let item = item
                .downcast::<ScreenObject>()
                .expect("Failed to downcast item");

            if item.app_name().eq(&ocr.app_name) {
                let dd_screen = &self.imp().dd_screen;
                dd_screen.set_model(Some(&list));
                dd_screen.set_selected(i);

                return Ok(());
            }
        }

        Err(anyhow::anyhow!(
            "Please open the application before running."
        ))
    }

    fn stop(&self) -> State {
        WindowManager::close_window(WINDOW_NAME);
        State::Stopped
    }

    fn configure_page(&self) {
        if self.current_state() == State::Paused {
            WindowManager::close_window(WINDOW_NAME);
            self.change_state(State::Stopped);
        } else {
            self.open_overlay_page(false);
            let areas = self.translation_areas();
            if areas.is_err() {
                return;
            }
            self.draw_rectagles(areas.unwrap().clone());
            self.change_state(State::Paused);
        }
    }

    fn change_state(&self, state: State) {
        let obj = self.imp();
        match state {
            State::Started => {
                obj.chk_full_screen.set_sensitive(false);
                obj.config_button.set_sensitive(false);
                obj.remove_button.set_sensitive(false);
                obj.action_button.set_label("Stop");
            }
            State::Stopped => {
                obj.chk_full_screen.set_sensitive(true);
                obj.config_button
                    .set_sensitive(!obj.chk_full_screen.is_active());
                obj.action_button.set_sensitive(true);
                obj.remove_button.set_sensitive(true);
                obj.action_button.set_label("Start");
                obj.config_button.set_label("Configure Translation Areas");
            }
            State::Paused => {
                obj.chk_full_screen.set_sensitive(false);
                obj.action_button.set_sensitive(false);
                obj.remove_button.set_sensitive(false);
                obj.config_button.set_label("Stop configuring");
            }
        }
        obj.state.replace(state);
    }

    fn text_overlay(&self) -> Result<()> {
        let obj = self.imp();
        obj.running.replace(true);
        obj.status_label.set_text("Running");

        let ocr = self.ocr_data()?;
        let is_vertical = ocr.is_vertical;
        let screen = self.screen_data()?;
        let translator = self.translator_data()?.code;
        let settings = self.settings();
        let provider = settings.tra_provider().to_string();
        let areas = self.translation_areas()?;
        let is_areas = !obj.chk_full_screen.is_active();

        let (tx, mut rx) = mpsc::channel(1);
        thread::spawn(move || {
            let rc = tokio::runtime::Runtime::new().unwrap();
            rc.block_on(async {
                let client = match translation::client().await {
                    Ok(value) => value,
                    Err(err) => {
                        let _ = tx.send(Err(anyhow::anyhow!(err.to_string()))).await;
                        return;
                    }
                };

                while !tx.is_closed() {
                    let areas = if is_areas {
                        ocr.ocr_areas(&areas, &screen)
                    } else {
                        ocr.ocr_screen(&screen)
                    };

                    if let Err(err) = areas {
                        let _ = tx.send(Err(anyhow::anyhow!(err.to_string()))).await;
                        return;
                    }
                    let mut areas = areas.unwrap();

                    if translator != "nt" {
                        for area in &mut areas {
                            sleep(Duration::from_millis(200)).await;
                            let result = match provider.as_str() {
                                "google" => {
                                    translation::google(
                                        &client,
                                        &area.text,
                                        &ocr.to_translator().code,
                                        &translator,
                                    )
                                    .await
                                }
                                _ => {
                                    translation::deepl(
                                        &client,
                                        &area.text.clone(),
                                        &ocr.to_translator().code,
                                        &translator.clone(),
                                    )
                                    .await
                                }
                            };
                            if let Err(err) = result {
                                let _ = tx.send(Err(anyhow::anyhow!(err.to_string()))).await;
                                return;
                            }
                            area.text = result.unwrap();
                        }
                    }

                    let _ = tx.send(Ok(areas)).await;
                    sleep(Duration::from_millis(TRANSLATION_DELAY)).await;
                }

                let _ = client.close().await;
            });
        });

        glib::spawn_future_local(clone!(@weak self as window => async move {
            while let Some(message) = rx.recv().await {
                if *window.imp().state.borrow() == State::Stopped { break; };
                let areas = match message {
                    Ok(value) => { value },
                    Err(err) => {
                        window.error_dialog(&err.to_string());
                        break;
                    }
                };
                let _ = window.draw_text(areas, is_vertical);
            }
            rx.close();
            window.imp().running.replace(false);
            window.imp().status_label.set_text("Idle");
        }));

        Ok(())
    }

    fn error_dialog(&self, message: &str) {
        self.change_state(self.stop());
        self.dialog("Text Overlay Error", message);
    }

    fn draw_text(&self, areas: Vec<AreaData>, vertical: bool) -> Result<()> {
        let regex = Regex::new(r"\n+").unwrap();

        let obj = self.imp();
        obj.drawing_area.queue_draw();
        obj.drawing_area
            .set_draw_func(move |_, cr, _width, _height| {
                cr.select_font_face(
                    "Sarasa Gothic J",
                    gtk::cairo::FontSlant::Normal,
                    gtk::cairo::FontWeight::Normal,
                );
                cr.set_antialias(gtk::cairo::Antialias::Fast);

                for area in areas.iter() {
                    regex.replace_all(&area.text, "\n").into_owned();
                    if area.text.trim().is_empty() {
                        continue;
                    }
                    if vertical {
                        draw_vertical_line(cr, area);
                    } else {
                        draw_line(cr, area);
                    }
                }
                cr.stroke().expect("Invalid cairo surface state");
            });
        Ok(())
    }
}

fn draw_vertical_line(cr: &gtk::cairo::Context, rect: &AreaData) {
    let mut x = rect.x as f64;
    let y = rect.y as f64;
    let lines = rect.text.lines();
    let font_size = utils::calc_font_size(&lines, rect.height, rect.width);

    cr.set_font_size(font_size);
    for line in lines {
        let mut _y = y;
        for c in line.split("") {
            draw_text_with_outline(cr, x, _y, c);
            _y += font_size;
        }
        x += font_size;
    }
}

fn draw_line(cr: &gtk::cairo::Context, rect: &AreaData) {
    let x = rect.x as f64;
    let mut y = rect.y as f64;
    let lines = rect.text.lines();
    let font_size = utils::calc_font_size(&lines, rect.width, rect.height);
    cr.set_font_size(font_size);
    for line in lines {
        y += font_size;
        draw_text_with_outline(cr, x, y, line);
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

fn open_windows() -> Result<ListStore> {
    let list = ListStore::new::<ScreenObject>();
    let windows = xcap::Window::all()?;
    for win in windows {
        if win.title().is_err() || win.title()?.is_empty() {
            continue;
        }
        let title = utils::truncate_string(&win.title()?, 70);

        list.append(&ScreenObject::new(win.id()?, win.app_name()?, title));
    }
    Ok(list)
}
