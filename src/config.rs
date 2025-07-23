// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the EUPL
//
// See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

use glib::{GStr, dpgettext2, gstr};
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

/// The full app license text.
pub const LICENSE_TEXT: &str = include_str!("../LICENSE");

pub fn license_text() -> String {
    dpgettext2(
        None,
        "about-dialog.license-text",
        // Translators: This is Pango markup, be sure to escape appropriately
        "Copyright Sebastian Wiesner &lt;sebastian@swsnr.de&gt;

Licensed under the terms of the EUPL 1.2. You can find official translations \
of the license text at <a href=\"%1\">%1</a>.

The full English text follows.

%2",
    )
    .replace(
        "%1",
        "https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12",
    )
    .replace("%2", &glib::markup_escape_text(LICENSE_TEXT))
}

/// Whether the app is running in flatpak.
fn running_in_flatpak() -> bool {
    std::fs::exists("/.flatpak-info").unwrap_or_default()
}

/// Whether this is a development or nightly build.
pub fn is_development() -> bool {
    APP_ID.ends_with(".Devel")
}

/// Get the locale directory.
///
/// Return the flatpak locale directory when running in flatpak.
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
