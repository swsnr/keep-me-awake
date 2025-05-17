// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

import GObject from "gi://GObject?version=2.0";
import Gtk from "gi://Gtk?version=4.0";
import Adw from "gi://Adw?version=1";

import { gettext as _, pgettext as C_ } from "gettext";

import * as config from "./config.js";

export const KMAApplication = GObject.registerClass(
  {
    GTypeName: "KMAApplication",
  },
  class extends Adw.Application {
    constructor() {
      super({
        application_id: config.APPID,
        resource_base_path: "/de/swsnr/keepmeawake",
      });
    }

    override vfunc_startup(): void {
      super.vfunc_startup();
      console.info(
        "Starting application, version",
        this.version,
        "development?",
        config.isDevelopment(),
      );

      Gtk.Window.set_default_icon_name(config.APPID);

      this.#setupActions();
    }

    #setupActions(): void {
      this.add_action_entries([
        {
          name: "quit",
          activate: () => {
            this.quit();
          },
        },
        {
          name: "about",
          activate: () => {
            this.#showAboutDialog();
          },
        },
      ]);

      this.set_accels_for_action("app.quit", ["<Control>q"]);
    }

    #showAboutDialog(): void {
      const dialog = Adw.AboutDialog.new_from_appdata(
        "/de/swsnr/keepmeawake/de.swsnr.keepmeawake.metainfo.xml",
        config.VERSION,
      );
      dialog.set_version(config.VERSION);

      dialog.add_link(
        C_("about-dialog.link.label", "Translations"),
        "https://translate.codeberg.org/engage/de-swsnr-keepmeawake/",
      );

      dialog.set_developers(["Sebastian Wiesner https://swsnr.de"]);
      dialog.set_designers(["Sebastian Wiesner https://swsnr.de"]);
      // Credits for the translator to the current language.
      // Translators: Add your name here, as "Jane Doe <jdoe@example.com>" or "Jane Doe https://jdoe.example.com"
      // Mail address or URL are optional.  Separate multiple translators with a newline, i.e. \n
      dialog.set_translator_credits(_("translator-credits"));
      dialog.add_acknowledgement_section(
        C_("about-dialog.acknowledgment-section", "Helpful services"),
        [
          "Codeberg https://codeberg.org",
          "Flathub https://flathub.org/",
          "Open Build Service https://build.opensuse.org/",
        ],
      );

      dialog.add_other_app(
        "de.swsnr.pictureoftheday",
        // Translators: Use app name from https://codeberg.org/swsnr/picture-of-the-day
        C_("about-dialog.other-app.name", "Picture Of The Day"),
        // Translators: Use summary from https://codeberg.org/swsnr/picture-of-the-day
        C_("about-dialog.other-app.summary", "Your daily wallpaper"),
      );
      dialog.add_other_app(
        "de.swsnr.turnon",
        // Translators: Use app name from https://codeberg.org/swsnr/turnon
        C_("about-dialog.other-app.name", "Turn On"),
        // Translators: Use summary from https://codeberg.org/swsnr/turnon
        C_("about-dialog.other-app.summary", "Turn on devices in your network"),
      );

      dialog.present(this.get_active_window());
    }

    async #createMainWindow(): Promise<Adw.ApplicationWindow> {
      // Asynchronously import the widgets module, to make sure that resources
      // are registered before the class object is created.
      const widgets = await import("./app/widgets.js");
      console.debug("Creating main application window");
      return new widgets.KMAApplicationWindow({ application: this });
    }

    override vfunc_activate(): void {
      super.vfunc_activate();
      console.debug("Activating application");

      const window = this.get_active_window();
      if (window) {
        window.present();
      } else {
        // Hold on to the application until we've loaded the main window class
        // asynchronously.
        this.hold();
        this.#createMainWindow()
          .finally(() => {
            // Release the hold
            this.release();
          })
          .then((window) => {
            window.present();
            if (config.isDevelopment()) {
              window.add_css_class("devel");
            }
          })
          .catch((error: unknown) => {
            console.error("Failed to create main window", error);
            this.quit();
          });
      }
    }
  },
);
