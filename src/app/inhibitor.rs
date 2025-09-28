// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the EUPL
//
// See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

use glib::Object;
use gtk::ApplicationInhibitFlags;

glib::wrapper! {
    pub struct Inhibitor(ObjectSubclass<imp::Inhibitor>);
}

impl Default for Inhibitor {
    fn default() -> Self {
        Object::builder().build()
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

impl Default for Inhibit {
    fn default() -> Self {
        Self::Nothing
    }
}

impl From<Inhibit> for ApplicationInhibitFlags {
    fn from(value: Inhibit) -> Self {
        match value {
            Inhibit::Nothing => ApplicationInhibitFlags::empty(),
            Inhibit::Suspend => ApplicationInhibitFlags::SUSPEND,
            Inhibit::SuspendAndIdle => {
                ApplicationInhibitFlags::SUSPEND | ApplicationInhibitFlags::IDLE
            }
        }
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::{Properties, WeakRef, dpgettext2};
    use gtk::ApplicationInhibitFlags;
    use gtk::gio::ApplicationHoldGuard;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::config::G_LOG_DOMAIN;

    use super::Inhibit;

    #[derive(Debug)]
    struct InhibitCookieGuard {
        app: WeakRef<gtk::Application>,
        flags: gtk::ApplicationInhibitFlags,
        cookie: u32,
    }

    impl InhibitCookieGuard {
        fn acquire(
            app: &impl IsA<gtk::Application>,
            flags: gtk::ApplicationInhibitFlags,
            reason: Option<&str>,
        ) -> Self {
            // We explicitly do not pass a window here, even if the application has an active one.
            //
            // If a window is given, Gtk inhibits idle on the compositor via the window surface.
            // But that's precisely not what we want: we wish to continue inhibiting even if the
            // window is closed.
            let cookie = app.inhibit(gtk::Window::NONE, flags, reason);
            glib::debug!("Acquired inhibit cookie {cookie} for {flags:?}");
            Self {
                app: app.as_ref().downgrade(),
                flags,
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

    #[derive(Debug)]
    enum InhibitState {
        Nothing,
        Inhibited(#[allow(dead_code)] ApplicationHoldGuard, InhibitCookieGuard),
    }

    impl From<&InhibitState> for Inhibit {
        fn from(value: &InhibitState) -> Self {
            match value {
                InhibitState::Nothing => Self::Nothing,
                InhibitState::Inhibited(_, cookie) => {
                    if cookie.flags.contains(ApplicationInhibitFlags::IDLE) {
                        Self::SuspendAndIdle
                    } else {
                        Self::Suspend
                    }
                }
            }
        }
    }

    impl Default for InhibitState {
        fn default() -> Self {
            Self::Nothing
        }
    }

    #[derive(Properties, Debug, Default)]
    #[properties(wrapper_type = super::Inhibitor)]
    pub struct Inhibitor {
        /// The application to inhibit on.
        #[property(get = Self::get_application, set = Self::set_application, type = Option<gtk::Application>, nullable)]
        application: RefCell<Option<WeakRef<gtk::Application>>>,
        /// Inhibitors
        #[property(explicit_notify, get = Self::get_inhibitors, set = Self::set_inhibitors, type = super::Inhibit, builder(super::Inhibit::default()))]
        inhibitors: RefCell<InhibitState>,
    }

    impl Inhibitor {
        fn get_application(&self) -> Option<gtk::Application> {
            self.application.borrow().as_ref()?.upgrade().clone()
        }

        fn set_application(&self, app: &gtk::Application) {
            self.application.replace(Some(app.downgrade()));
            // Clear inhibitors of the old app
            self.set_inhibitors(Inhibit::Nothing);
        }

        fn get_inhibitors(&self) -> Inhibit {
            Inhibit::from(&*self.inhibitors.borrow())
        }

        fn set_inhibitors(&self, inhibit: Inhibit) {
            if self.get_inhibitors() == inhibit {
                return;
            }
            glib::info!("Inhibiting: {inhibit:?}");
            let new_state = match inhibit {
                Inhibit::Nothing => InhibitState::Nothing,
                Inhibit::Suspend | Inhibit::SuspendAndIdle => {
                    if let Some(application) = self.get_application() {
                        let cookie = InhibitCookieGuard::acquire(
                            &application,
                            inhibit.into(),
                            Some(&dpgettext2(
                                None,
                                "inhibit-reason",
                                "Keep Me Awake inhibits suspend at your request.",
                            )),
                        );
                        InhibitState::Inhibited(application.hold(), cookie)
                    } else {
                        glib::warn!("Cannot inhibit with application reference");
                        InhibitState::Nothing
                    }
                }
            };
            self.inhibitors.replace(new_state);
            self.obj().notify_inhibitors();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Inhibitor {
        const NAME: &'static str = "KeepMeAwakeInhibitor";

        type Type = super::Inhibitor;

        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Inhibitor {}
}
