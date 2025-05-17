// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

import Gio from "gi://Gio?version=2.0";

/**
 * Application ID for this application.
 */
export const APPID = "@APPID@";

/**
 * The version of the application.
 */
export const VERSION = "@VERSION@";

/**
 * GLib logging domain for this application.
 */
export const G_LOG_DOMAIN = "KeepMeAwake";

/**
 * Whether the application runs within flatpak.
 *
 * @returns `true` if running inside flatpak, `false` otherwise.
 */
export const runningInFlatpak = (): boolean =>
  Gio.File.new_for_path("/.flatpak-info").query_exists(null);

/**
 * Whether this is a development build.
 *
 * It's a development build if the app ID ends with `.Devel`.
 *
 * @returns `true` if a development build, `false` otherwise.
 */
export const isDevelopment = (): boolean => APPID.endsWith(".Devel");

/**
 * Load and register all resources of this application.
 */
export const loadAndRegisterResources = () => {
  // The app modules are in the js directory, so we need to go to levels up and then down into resuorces
  const resourcesDirectory = Gio.File.new_for_uri(import.meta.url)
    .get_parent()
    ?.get_parent()
    ?.get_child("resources");
  for (const resourceName of ["resources.generated.gresource"]) {
    const resourceFile = resourcesDirectory?.get_child(resourceName).get_path();
    console.debug("Loading and registering resource", resourceFile);
    if (resourceFile) {
      const resource = Gio.Resource.load(resourceFile);
      resource._register();
    } else {
      throw new Error("Failed to determine path of resource file");
    }
  }
};
