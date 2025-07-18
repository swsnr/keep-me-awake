// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the EUPL
//
// See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

use glib::{WeakRef, object::IsA};
use gtk::gio::ApplicationHoldGuard;
use gtk::prelude::*;

use crate::config::G_LOG_DOMAIN;

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
pub struct InhibitCookieGuard {
    app: WeakRef<gtk::Application>,
    cookie: u32,
}

impl InhibitCookieGuard {
    pub fn acquire(
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
pub enum InhibitState {
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
