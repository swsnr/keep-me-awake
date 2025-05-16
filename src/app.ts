// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

import GObject from "gi://GObject?version=2.0";
import Gtk from "gi://Gtk?version=4.0";
import Adw from "gi://Adw?version=1";

import * as config from "./config.js";

export const KMAApplication = GObject.registerClass(
  {
    GTypeName: "KMAApplication",
  },
  class extends Adw.Application {
    override vfunc_startup(): void {
      super.vfunc_startup();
      console.info(
        "Starting application, version",
        this.version,
        "development?",
        config.isDevelopment(),
      );

      Gtk.Window.set_default_icon_name(config.APPID);
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
