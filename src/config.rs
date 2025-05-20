// Copyright Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use glib::{GStr, gstr};
use gtk::gio::{self, resources_register};

pub const APP_ID: &GStr =
    // SAFETY: We explicitly append a nul byte
    unsafe {
        GStr::from_str_with_nul_unchecked(concat!(include_str!("../build/app-id"), "\0"))
    };

pub const G_LOG_DOMAIN: &str = "KeepMeAwake";

/// The Cargo package verson.
///
/// This provides the full version from `Cargo.toml`.
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Whether the app is running in flatpak.
fn running_in_flatpak() -> bool {
    std::fs::exists("/.flatpak-info").unwrap_or_default()
}

/// Get the locale directory.
///
/// Return the flatpak locale directory when in
pub fn locale_directory() -> &'static GStr {
    if running_in_flatpak() {
        gstr!("/app/share/locale")
    } else {
        gstr!("/usr/share/locale")
    }
}

/// Load and register resource files from manifest directory in a debug build.
#[cfg(debug_assertions)]
pub fn register_resources() {
    // In a debug build load resources from a file
    let files = [
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "build/resources/resources.generated.gresource"
        ),
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/",
            "build/resources/resources.data.gresource"
        ),
    ];
    for file in files {
        let resource =
            gio::Resource::load(file).expect("Fail to load resource, run 'just compile'!");
        resources_register(&resource);
    }
}

/// Register embedded resource data in a release build.
#[cfg(not(debug_assertions))]
pub fn register_resources() {
    let generated = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/",
        "build/resources/resources.generated.gresource"
    ));
    let data = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/",
        "build/resources/resources.data.gresource"
    ));
    for resource in [generated.as_slice(), data.as_slice()] {
        let bytes = glib::Bytes::from_static(resource);
        let resource = gio::Resource::from_data(&bytes).unwrap();
        resources_register(&resource);
    }
}
