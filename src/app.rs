// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the EUPL
//
// See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

use adw::prelude::*;
use glib::{Object, dgettext, dpgettext2};
use gtk::gio::ActionEntry;

mod widgets;

glib::wrapper! {
    pub struct KeepMeAwakeApplication(ObjectSubclass<imp::KeepMeAwakeApplication>)
        @extends adw::Application, gtk::Application, gtk::gio::Application,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap;
}

impl KeepMeAwakeApplication {
    fn show_about_dialog(&self) {
        let dialog = adw::AboutDialog::from_appdata(
            "/de/swsnr/keepmeawake/de.swsnr.keepmeawake.metainfo.xml",
            Some(crate::config::CARGO_PKG_VERSION),
        );
        dialog.set_version(crate::config::CARGO_PKG_VERSION);

        dialog.add_link(
            &dpgettext2(None, "about-dialog.link.label", "Translations"),
            "https://translate.codeberg.org/engage/de-swsnr-keepmeawake/",
        );

        dialog.set_developers(&["Sebastian Wiesner https://swsnr.de"]);
        dialog.set_designers(&["Sebastian Wiesner https://swsnr.de"]);
        // Credits for the translator to the current language.
        // Translators: Add your name here, as "Jane Doe <jdoe@example.com>" or "Jane Doe https://jdoe.example.com"
        // Mail address or URL are optional.  Separate multiple translators with a newline, i.e. \n
        dialog.set_translator_credits(&dgettext(None, "translator-credits"));
        dialog.add_acknowledgement_section(
            Some(&dpgettext2(
                None,
                "about-dialog.acknowledgment-section",
                "Helpful services",
            )),
            &[
                "Codeberg https://codeberg.org",
                "Flathub https://flathub.org/",
                "Open Build Service https://build.opensuse.org/",
            ],
        );

        dialog.add_other_app(
            "de.swsnr.pictureoftheday",
            // Translators: Use app name from https://codeberg.org/swsnr/picture-of-the-day
            &dpgettext2(None, "about-dialog.other-app.name", "Picture Of The Day"),
            &dpgettext2(
                None,
                "about-dialog.other-app.summary",
                // Translators: Use summary from https://codeberg.org/swsnr/picture-of-the-day
                "Your daily wallpaper",
            ),
        );
        dialog.add_other_app(
            "de.swsnr.turnon",
            // Translators: Use app name from https://codeberg.org/swsnr/turnon
            &dpgettext2(None, "about-dialog.other-app.name", "Turn On"),
            &dpgettext2(
                None,
                "about-dialog.other-app.summary",
                // Translators: Use summary from https://codeberg.org/swsnr/turnon
                "Turn on devices in your network",
            ),
        );
        dialog.present(self.active_window().as_ref());
    }

    fn setup_actions(&self) {
        let entries = [
            ActionEntry::builder("quit")
                .activate(|app: &KeepMeAwakeApplication, _, _| app.quit())
                .build(),
            ActionEntry::builder("about")
                .activate(|app: &KeepMeAwakeApplication, _, _| {
                    app.show_about_dialog();
                })
                .build(),
        ];
        self.add_action_entries(entries);
        self.set_accels_for_action("app.action", &["<Control>q"]);
    }
}

impl Default for KeepMeAwakeApplication {
    fn default() -> Self {
        Object::builder()
            .property("application-id", crate::config::APP_ID)
            .property("resource-base-path", "/de/swsnr/keepmeawake")
            .build()
    }
}

mod imp {
    use adw::subclass::prelude::*;
    use glib::object::Cast;
    use gtk::prelude::{GtkApplicationExt, GtkWindowExt};

    use crate::config::{self, G_LOG_DOMAIN};

    use super::widgets::KeepMeAwakeApplicationWindow;

    #[derive(Default)]
    pub struct KeepMeAwakeApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for KeepMeAwakeApplication {
        const NAME: &'static str = "KMAApplication";

        type Type = super::KeepMeAwakeApplication;

        type ParentType = adw::Application;
    }

    impl ObjectImpl for KeepMeAwakeApplication {}

    impl ApplicationImpl for KeepMeAwakeApplication {
        fn startup(&self) {
            self.parent_startup();

            glib::info!("Starting application {}", config::CARGO_PKG_VERSION);
            gtk::Window::set_default_icon_name(config::APP_ID);

            self.obj().setup_actions();
        }

        fn activate(&self) {
            let window = self
                .obj()
                .active_window()
                .unwrap_or_else(|| KeepMeAwakeApplicationWindow::new(&*self.obj()).upcast());
            window.present();
        }
    }

    impl GtkApplicationImpl for KeepMeAwakeApplication {}

    impl AdwApplicationImpl for KeepMeAwakeApplication {}
}
