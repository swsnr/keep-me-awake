// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the EUPL
//
// See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

use adw::prelude::*;
use glib::{Object, dgettext, dpgettext2};
use gnome_app_utils::portal::{
    background::{RequestBackgroundFlags, request_background},
    window::PortalWindowHandle,
};
use gtk::gio::{ActionEntry, IOErrorEnum, PropertyAction};

use crate::config::G_LOG_DOMAIN;

use inhibit::Inhibit;

mod inhibit;
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

        dialog.set_license_type(gtk::License::Custom);
        dialog.set_license(&crate::config::license_text());

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
            // Translators: Use app name from https://flathub.org/apps/de.swsnr.pictureoftheday
            &dpgettext2(None, "about-dialog.other-app.name", "Picture Of The Day"),
            &dpgettext2(
                None,
                "about-dialog.other-app.summary",
                // Translators: Use summary from https://flathub.org/apps/de.swsnr.pictureoftheday
                "Your daily wallpaper",
            ),
        );
        dialog.add_other_app(
            "de.swsnr.turnon",
            // Translators: Use app name from https://flathub.org/apps/de.swsnr.turnon
            &dpgettext2(None, "about-dialog.other-app.name", "Turn On"),
            &dpgettext2(
                None,
                "about-dialog.other-app.summary",
                // Translators: Use summary from https://flathub.org/apps/de.swsnr.turnon
                "Turn on devices in your network",
            ),
        );
        dialog.present(self.active_window().as_ref());
    }

    fn setup_actions(&self) {
        let entries = [
            ActionEntry::builder("quit")
                .activate(|app: &KeepMeAwakeApplication, _, _| {
                    // Clear inhibitor to withdraw notifications and release the
                    // inhibition app hold.  Do this first to avoid showing any
                    // new notifications when closing the main window next.
                    app.set_inhibitors(Inhibit::Nothing);
                    // Close the main window to release its app hold.  With no
                    // holds left the app will automatically exit on its idle
                    // timeout.
                    if let Some(window) = app.active_window() {
                        window.close();
                    }
                })
                .build(),
            ActionEntry::builder("about")
                .activate(|app: &KeepMeAwakeApplication, _, _| {
                    app.show_about_dialog();
                })
                .build(),
        ];
        self.add_action_entries(entries);

        self.add_action(&PropertyAction::new("inhibit", self, "inhibitors"));

        self.set_accels_for_action("app.quit", &["<Control>q"]);
    }

    fn toggle_keep_me_awake(&self) {
        let new_inhibitor: Inhibit = match self.inhibitors() {
            Inhibit::Nothing => Inhibit::SuspendAndIdle,
            Inhibit::Suspend | Inhibit::SuspendAndIdle => Inhibit::Nothing,
        };
        self.set_inhibitors(new_inhibitor);
    }

    async fn ask_background(&self) -> Result<(), glib::Error> {
        let connection = self.dbus_connection().unwrap();
        let reason = dpgettext2(
            None,
            "portal.request-background.reason",
            "Inhibit suspend and idle without a main window",
        );
        let parent_window = PortalWindowHandle::new_for_app(self).await;
        glib::info!("Requesting permission to run in background");
        let result = request_background(
            &connection,
            &parent_window,
            Some(&reason),
            None,
            RequestBackgroundFlags::empty(),
        )
        .await?;

        if result.background {
            Ok(())
        } else {
            Err(glib::Error::new(
                IOErrorEnum::Failed,
                "Background permission not granted",
            ))
        }
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
    use std::{cell::RefCell, rc::Rc};

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use futures_util::{StreamExt, future};
    use glib::dpgettext2;
    use gnome_app_utils::{
        app::AppUpdatedMonitor,
        portal::{
            PortalSession,
            global_shortcuts::{GlobalShortcutsSession, NewShortcut},
            window::PortalWindowHandle,
        },
    };
    use gtk::{
        ApplicationInhibitFlags,
        gio::Notification,
        prelude::{GtkApplicationExt, GtkWindowExt},
    };

    use crate::{
        app::inhibit::InhibitCookieGuard,
        config::{self, G_LOG_DOMAIN},
    };

    use super::{
        inhibit::{Inhibit, InhibitState},
        widgets::KeepMeAwakeApplicationWindow,
    };

    const NOTIFICATION_ID: &str = "de.swsnr.keepmeawake.persistent-inhibitor-notification";

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::KeepMeAwakeApplication)]
    pub struct KeepMeAwakeApplication {
        #[property(explicit_notify, get = Self::get_inhibitors, set = Self::set_inhibitors, type = super::Inhibit, builder(super::Inhibit::default()))]
        inhibitors: RefCell<InhibitState>,
        /// App updates monitor,
        updated_monitor: AppUpdatedMonitor,
        /// Session for global shortcuts
        shortcuts_session: RefCell<Option<Rc<GlobalShortcutsSession>>>,
    }

    impl KeepMeAwakeApplication {
        fn get_inhibitors(&self) -> Inhibit {
            Inhibit::from(&*self.inhibitors.borrow())
        }

        fn set_inhibitors(&self, inhibit: Inhibit) {
            if self.get_inhibitors() == inhibit {
                return;
            }
            let new_state = match inhibit {
                Inhibit::Nothing => {
                    glib::info!("Inhibiting nothing");
                    InhibitState::Nothing
                }
                Inhibit::Suspend => {
                    glib::info!("Inhibiting suspend");
                    InhibitState::Suspend(
                        self.obj().hold(),
                        InhibitCookieGuard::acquire(
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
                Inhibit::SuspendAndIdle => {
                    glib::info!("Inhibiting suspend and idle");
                    InhibitState::SuspendAndIdle(
                        self.obj().hold(),
                        InhibitCookieGuard::acquire(
                            &*self.obj(),
                            ApplicationInhibitFlags::SUSPEND | ApplicationInhibitFlags::IDLE,
                            Some(&dpgettext2(
                                None,
                                "inhibit-reason",
                                "Keep Me Awake inhibits suspend and screen lock at your request.",
                            )),
                        ),
                    )
                }
            };
            self.inhibitors.replace(new_state);
            self.obj().notify_inhibitors();
            self.update_notification(inhibit);
        }

        fn update_notification(&self, inhibit: Inhibit) {
            let notification = match inhibit {
                Inhibit::Nothing => None,
                Inhibit::Suspend => {
                    let notification = Notification::new(&dpgettext2(
                        None,
                        "notification.title",
                        "Suspend inhibited",
                    ));
                    notification.set_body(Some(&dpgettext2(
                        None,
                        "notification.body",
                        "Keep Me Awake inhibits suspend at your request.",
                    )));
                    Some(notification)
                }
                Inhibit::SuspendAndIdle => {
                    let notification = Notification::new(&dpgettext2(
                        None,
                        "notification.title",
                        "Suspend and screen lock inhibited",
                    ));
                    notification.set_body(Some(&dpgettext2(
                        None,
                        "notification.body",
                        "Keep Me Awake inhibits suspend and screen lock at your request.",
                    )));
                    Some(notification)
                }
            };
            if let Some(notification) = notification {
                self.obj()
                    .send_notification(Some(NOTIFICATION_ID), &notification);
            } else {
                self.obj().withdraw_notification(NOTIFICATION_ID);
            }
        }

        async fn setup_global_shortcuts(&self) -> Result<(), glib::Error> {
            if self.shortcuts_session.borrow().is_some() {
                return Ok(());
            }

            let connection = self.obj().dbus_connection().unwrap();
            glib::info!("Creating session for global shortcuts");
            let session = Rc::new(GlobalShortcutsSession::create(&connection).await?);
            self.shortcuts_session.replace(Some(session.clone()));

            glib::info!(
                "Binding global shortcuts in session {}",
                session.object_path().as_str()
            );
            let parent_window = PortalWindowHandle::new_for_app(&*self.obj()).await;
            // See https://specifications.freedesktop.org/shortcuts-spec/latest/ for shortcuts syntax.
            // Yes, the Super key is LOGO in this spec.
            session
                .bind_shortcuts(
                    &parent_window,
                    &[NewShortcut {
                        id: "keep-me-awake-toggle",
                        description: &dpgettext2(
                            None,
                            "global shortcut description",
                            "Toggle Keep Me Awake",
                        ),
                        preferred_trigger: Some("LOGO+w"),
                    }],
                )
                .await?;
            glib::spawn_future_local(session.receive_activated().for_each(glib::clone!(
                #[weak(rename_to = app)]
                self.obj(),
                #[upgrade_or]
                future::ready(()),
                move |activated| {
                    match activated.shortcut_id.as_str() {
                        "keep-me-awake-toggle" => {
                            glib::debug!("Toggling keep me awake by global shortcut");
                            app.toggle_keep_me_awake();
                        }
                        unknown => {
                            glib::warn!(
                                "Received activation signal for unknown global shortcut: {unknown}"
                            );
                        }
                    }
                    future::ready(())
                }
            )));
            Ok(())
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
            if let Some(window) = self.obj().active_window() {
                window.present();
            } else {
                let window = KeepMeAwakeApplicationWindow::new(&*self.obj());
                self.obj()
                    .bind_property("inhibitors", &window, "inhibitors")
                    .bidirectional()
                    .sync_create()
                    .build();

                // Re-display notification if the main window is closed, to make
                // sure the user doesn't forget.
                window.connect_close_request(glib::clone!(
                    #[weak_allow_none(rename_to = app)]
                    self.obj(),
                    move |_| {
                        if let Some(app) = app {
                            app.imp().update_notification(app.inhibitors());
                        }
                        glib::Propagation::Proceed
                    }
                ));

                if crate::config::is_development() {
                    window.add_css_class("devel");
                }
                window.present();

                self.updated_monitor
                    .bind_property("updated", &window, "show-update-indicator")
                    .sync_create()
                    .build();

                // Request background if the app gets activated, to be able to
                // keep running while inhibiting even if the user closes the window.
                // Do so after the window was presented to make there's a window to
                // show permission prompts on.
                glib::spawn_future_local(glib::clone!(
                    #[weak(rename_to = app)]
                    self.obj(),
                    async move {
                        if let Err(error) = app.ask_background().await {
                            glib::warn!("Background permission request failed: {}", error);
                        }
                        if let Err(error) = app.imp().setup_global_shortcuts().await {
                            glib::error!("Failed to setup global shortcuts: {error}");
                        }
                    }
                ));
            }
        }
    }

    impl GtkApplicationImpl for KeepMeAwakeApplication {}

    impl AdwApplicationImpl for KeepMeAwakeApplication {}
}
