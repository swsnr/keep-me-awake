// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the EUPL
//
// See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

use std::collections::HashMap;

use adw::prelude::*;
use glib::{Object, dgettext, dpgettext2, subclass::types::ObjectSubclassIsExt as _};
use gnome_app_utils::portal::{
    background::{RequestBackgroundFlags, request_background},
    window::PortalWindowHandle,
};
use gtk::gio::{ActionEntry, IOErrorEnum, PropertyAction, SimpleAction};

use crate::config::G_LOG_DOMAIN;

use inhibitor::Inhibit;

mod inhibitor;
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

    fn show_shortcuts_dialog(&self) {
        let builder = gtk::Builder::new();
        builder
            .add_from_resource(&format!(
                "{}/ui/shortcuts-dialog.ui",
                self.resource_base_path().unwrap()
            ))
            .unwrap();
        let dialog = builder
            .object::<adw::ShortcutsDialog>("shortcuts_dialog")
            .unwrap();
        let global_shortcuts = builder
            .object::<adw::ShortcutsSection>("global_shortcuts")
            .unwrap()
            .downgrade();

        if let Some(shortcuts_session) = self.imp().global_shortcuts_session() {
            glib::spawn_future_local(async move {
                match shortcuts_session.list_shortcuts().await {
                    Ok(shortcuts) => {
                        if let Some(global_shortcuts) = global_shortcuts.upgrade() {
                            let mut original_shortcuts = HashMap::new();
                            original_shortcuts.insert("keep-me-awake-toggle", "<Super>w");
                            for shortcut in &shortcuts {
                                let item = adw::ShortcutsItem::new(
                                    &shortcut.description,
                                    original_shortcuts
                                        .get(shortcut.id.as_str())
                                        .copied()
                                        .unwrap_or_default(),
                                );
                                item.set_subtitle(&shortcut.trigger_description);
                                global_shortcuts.add(item);
                            }
                        }
                    }
                    Err(error) => {
                        glib::error!("Failed to list shortcuts: {error}");
                    }
                }
            });
        }

        dialog.present(self.active_window().as_ref());
    }

    fn setup_actions(&self) {
        let entries = [
            ActionEntry::builder("configure-global-shortcuts")
                .activate(|app: &KeepMeAwakeApplication, _, _| {
                    let app = app.clone();
                    glib::spawn_future_local(async move {
                        app.imp().configure_global_shortcuts().await;
                    });
                })
                .build(),
            ActionEntry::builder("toggle-inhibit")
                .activate(|app: &KeepMeAwakeApplication, _, _| {
                    let inhibitor = app.imp().inhibitor();
                    let new_inhibitor: Inhibit = match inhibitor.inhibitors() {
                        Inhibit::Nothing => Inhibit::SuspendAndIdle,
                        Inhibit::Suspend | Inhibit::SuspendAndIdle => Inhibit::Nothing,
                    };
                    inhibitor.set_inhibitors(new_inhibitor);
                })
                .build(),
            ActionEntry::builder("shortcuts")
                .activate(|app: &KeepMeAwakeApplication, _, _| {
                    app.show_shortcuts_dialog();
                })
                .build(),
            ActionEntry::builder("quit")
                .activate(|app: &KeepMeAwakeApplication, _, _| {
                    // Clear out global shortcuts to drop hold guard for global shortcuts.
                    app.imp().drop_global_shortcuts();
                    // Clear inhibitor to withdraw notifications and release the
                    // inhibition app hold.  Do this first to avoid showing any
                    // new notifications when closing the main window next.
                    app.imp().inhibitor().set_inhibitors(Inhibit::Nothing);
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
        self.add_action(&PropertyAction::new(
            "inhibit",
            self.imp().inhibitor(),
            "inhibitors",
        ));

        // Disable action to configure global shortcuts until we've checked the
        // portal version.
        let configure_global_shortcuts = self
            .lookup_action("configure-global-shortcuts")
            .unwrap()
            .downcast::<SimpleAction>()
            .unwrap();
        configure_global_shortcuts.set_enabled(false);

        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("app.shortcuts", &["<Control>question"]);
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
            global_shortcuts::{
                ActivationToken, GlobalShortcutsSession, NewShortcut, global_shortcuts_version,
            },
            window::PortalWindowHandle,
        },
    };
    use gtk::gio::{ApplicationHoldGuard, Notification, SimpleAction};

    use crate::config::{APP_ID, CARGO_PKG_VERSION, G_LOG_DOMAIN};

    use super::inhibitor::{Inhibit, Inhibitor};
    use super::widgets::KeepMeAwakeApplicationWindow;

    const NOTIFICATION_ID: &str = "de.swsnr.keepmeawake.persistent-inhibitor-notification";

    #[derive(Default)]
    pub struct KeepMeAwakeApplication {
        inhibitor: Inhibitor,
        /// App updates monitor,
        updated_monitor: AppUpdatedMonitor,
        /// Session for global shortcuts
        global_shortcuts_session: RefCell<Option<Rc<GlobalShortcutsSession>>>,
        /// App hold guard, to keep running when we've set up global shortcuts.
        global_shortcuts_guard: RefCell<Option<ApplicationHoldGuard>>,
        /// The running task for consuming activated shortcuts
        global_shortcuts_activated_task: RefCell<Option<glib::JoinHandle<()>>>,
    }

    impl KeepMeAwakeApplication {
        pub fn inhibitor(&self) -> &Inhibitor {
            &self.inhibitor
        }

        pub fn global_shortcuts_session(&self) -> Option<Rc<GlobalShortcutsSession>> {
            self.global_shortcuts_session
                .borrow()
                .as_ref()
                .map(Clone::clone)
        }

        /// Drop global shortcuts session and app guard.
        pub fn drop_global_shortcuts(&self) {
            if let Some(handle) = self.global_shortcuts_activated_task.borrow_mut().take() {
                handle.abort();
            }
            self.global_shortcuts_session.borrow_mut().take();
            self.global_shortcuts_guard.borrow_mut().take();
        }

        pub async fn configure_global_shortcuts(&self) {
            if let Some(session) = self.global_shortcuts_session() {
                let parent_window = PortalWindowHandle::new_for_app(&*self.obj()).await;
                let activation_token = self
                    .obj()
                    .active_window()
                    .as_ref()
                    .and_then(ActivationToken::from_widget);
                if let Err(error) = session
                    .configure_shortcuts(&parent_window, activation_token.as_ref())
                    .await
                {
                    glib::error!("Failed to configure global shortcuts: {error}");
                }
            }
        }

        fn update_notification(&self) {
            let notification = match self.inhibitor.inhibitors() {
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
                        "Keep me Awake inhibits suspend at your request.",
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
                        "Keep me Awake inhibits suspend and screen lock at your request.",
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
            if self.global_shortcuts_session.borrow().is_some() {
                return Ok(());
            }

            let connection = self.obj().dbus_connection().unwrap();
            let version = global_shortcuts_version(&connection).await?;

            if version < 2 {
                glib::warn!(
                    "Global shortcuts portal version {version} found, disabling configuration of global shortcuts"
                );
            } else {
                self.obj()
                    .lookup_action("configure-global-shortcuts")
                    .unwrap()
                    .downcast::<SimpleAction>()
                    .unwrap()
                    .set_enabled(true);
            }

            glib::info!("Creating session for global shortcuts version {version}");
            let session = Rc::new(GlobalShortcutsSession::create(&connection).await?);
            self.global_shortcuts_session.replace(Some(session.clone()));

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
                            "Toggle Keep me Awake",
                        ),
                        preferred_trigger: Some("LOGO+w"),
                    }],
                )
                .await?;
            let handle = glib::spawn_future_local(session.receive_activated().for_each(glib::clone!(
                #[weak(rename_to = app)]
                self.obj(),
                #[upgrade_or]
                future::ready(()),
                move |activated| {
                    match activated.shortcut_id.as_str() {
                        "keep-me-awake-toggle" => {
                            glib::debug!("Toggling keep me awake by global shortcut");
                            app.activate_action("toggle-inhibit", None);
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
            self.global_shortcuts_activated_task.replace(Some(handle));
            Ok(())
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KeepMeAwakeApplication {
        const NAME: &'static str = "KeepMeAwakeApplication";

        type Type = super::KeepMeAwakeApplication;

        type ParentType = adw::Application;
    }

    impl ObjectImpl for KeepMeAwakeApplication {
        fn constructed(&self) {
            self.parent_constructed();

            self.inhibitor
                .set_application(Some(self.obj().as_ref().upcast_ref()));
            self.inhibitor.connect_inhibitors_notify(glib::clone!(
                #[weak(rename_to = app)]
                self.obj(),
                move |_| {
                    app.imp().update_notification();
                }
            ));
        }
    }

    impl ApplicationImpl for KeepMeAwakeApplication {
        fn startup(&self) {
            self.parent_startup();

            glib::info!("Starting application {}", CARGO_PKG_VERSION);
            gtk::Window::set_default_icon_name(APP_ID);

            self.obj().setup_actions();
        }

        fn activate(&self) {
            if let Some(window) = self.obj().active_window() {
                window.present();
            } else {
                let window = KeepMeAwakeApplicationWindow::new(&*self.obj());
                self.inhibitor
                    .bind_property("inhibitors", &window, "inhibitors")
                    .sync_create()
                    .build();

                // Re-display notification if the main window is closed, to make
                // sure the user doesn't forget.
                window.connect_close_request(glib::clone!(
                    #[weak(rename_to = app)]
                    self.obj(),
                    #[upgrade_or]
                    glib::Propagation::Proceed,
                    move |_| {
                        app.imp().update_notification();
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

                self.obj()
                    .activate_action("inhibit", Some(&"suspend-and-idle".to_variant()));

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
                        } else {
                            glib::info!("Global shortcuts set up, keep running in background");
                            app.imp().global_shortcuts_guard.replace(Some(app.hold()));
                        }
                    }
                ));
            }
        }
    }

    impl GtkApplicationImpl for KeepMeAwakeApplication {}

    impl AdwApplicationImpl for KeepMeAwakeApplication {}
}
