// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the EUPL
//
// See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

use adw::prelude::*;
use glib::{Object, WeakRef, dgettext, dpgettext2};
use gtk::gio::{ActionEntry, ApplicationHoldGuard};

use crate::config::G_LOG_DOMAIN;

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

/// What's currently being inhibited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "KeepMeAwakeInhibit")]
pub enum Inhibit {
    /// Inhibit nothing.
    Nothing,
    /// Inhibit suspend.
    Suspend,
    /// Inhibit suspend and session idle.
    SuspendAndIdle,
}

impl From<&InhibitState> for Inhibit {
    fn from(value: &InhibitState) -> Self {
        match value {
            InhibitState::Nothing => Self::Nothing,
            InhibitState::Suspend { .. } => Self::Suspend,
            InhibitState::SuspendAndIdle { .. } => Self::SuspendAndIdle,
        }
    }
}

impl Default for Inhibit {
    fn default() -> Self {
        Self::Nothing
    }
}

#[derive(Debug)]
struct InhibitCookieGuard {
    app: WeakRef<gtk::Application>,
    cookie: u32,
}

impl InhibitCookieGuard {
    fn acquire(
        app: &impl IsA<gtk::Application>,
        flags: gtk::ApplicationInhibitFlags,
        reason: Option<&str>,
    ) -> Self {
        let cookie = app.inhibit(app.active_window().as_ref(), flags, reason);
        glib::debug!("Acquired inhibit cookie {cookie} for {flags:?}");
        Self {
            app: app.as_ref().downgrade(),
            cookie,
        }
    }
}

impl Drop for InhibitCookieGuard {
    fn drop(&mut self) {
        if let Some(app) = self.app.upgrade() {
            glib::debug!("Dropping inhibit cookie {}", self.cookie);
            app.uninhibit(self.cookie);
            self.cookie = 0;
        }
    }
}

enum InhibitState {
    Nothing,
    // we just store these to keep them until they are dropped
    #[allow(dead_code)]
    Suspend(ApplicationHoldGuard, InhibitCookieGuard),
    #[allow(dead_code)]
    SuspendAndIdle(ApplicationHoldGuard, InhibitCookieGuard),
}

impl Default for InhibitState {
    fn default() -> Self {
        Self::Nothing
    }
}

mod imp {
    use std::cell::RefCell;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use glib::{dpgettext2, object::Cast};
    use gtk::{
        ApplicationInhibitFlags,
        prelude::{GtkApplicationExt, GtkWindowExt},
    };

    use crate::config::{self, G_LOG_DOMAIN};

    use super::{InhibitState, widgets::KeepMeAwakeApplicationWindow};

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::KeepMeAwakeApplication)]
    pub struct KeepMeAwakeApplication {
        #[property(get = Self::get_inhibitors, set = Self::set_inhibitors, type = super::Inhibit, builder(super::Inhibit::default()))]
        inhibitors: RefCell<InhibitState>,
    }

    impl KeepMeAwakeApplication {
        fn get_inhibitors(&self) -> super::Inhibit {
            super::Inhibit::from(&*self.inhibitors.borrow())
        }

        fn set_inhibitors(&self, inhibit: super::Inhibit) {
            let new_state = match inhibit {
                super::Inhibit::Nothing => {
                    glib::info!("Inhibiting nothing");
                    super::InhibitState::Nothing
                }
                super::Inhibit::Suspend => {
                    glib::info!("Inhibiting suspend");
                    super::InhibitState::Suspend(
                        self.obj().hold(),
                        super::InhibitCookieGuard::acquire(
                            &*self.obj(),
                            ApplicationInhibitFlags::SUSPEND,
                            Some(&dpgettext2(
                                None,
                                "inhibit-reason",
                                "Keep Me Awake inhibits suspend at your request.",
                            )),
                        ),
                    )
                }
                super::Inhibit::SuspendAndIdle => {
                    glib::info!("Inhibiting suspend and idle");
                    super::InhibitState::SuspendAndIdle(
                        self.obj().hold(),
                        super::InhibitCookieGuard::acquire(
                            &*self.obj(),
                            ApplicationInhibitFlags::SUSPEND | ApplicationInhibitFlags::IDLE,
                            Some(&dpgettext2(
                                None,
                                "inhibit-reason",
                                "Keep Me Awake inhibits suspend and idle at your request.",
                            )),
                        ),
                    )
                }
            };
            self.inhibitors.replace(new_state);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KeepMeAwakeApplication {
        const NAME: &'static str = "KeepMeAwakeApplication";

        type Type = super::KeepMeAwakeApplication;

        type ParentType = adw::Application;
    }

    #[glib::derived_properties]
    impl ObjectImpl for KeepMeAwakeApplication {}

    impl ApplicationImpl for KeepMeAwakeApplication {
        fn startup(&self) {
            self.parent_startup();

            glib::info!("Starting application {}", config::CARGO_PKG_VERSION);
            gtk::Window::set_default_icon_name(config::APP_ID);

            self.obj().setup_actions();
        }

        fn activate(&self) {
            let window = self.obj().active_window().unwrap_or_else(|| {
                let window = KeepMeAwakeApplicationWindow::new(&*self.obj());
                self.obj()
                    .bind_property("inhibitors", &window, "inhibitors")
                    .bidirectional()
                    .sync_create()
                    .build();
                window.upcast()
            });
            window.present();
        }
    }

    impl GtkApplicationImpl for KeepMeAwakeApplication {}

    impl AdwApplicationImpl for KeepMeAwakeApplication {}
}
