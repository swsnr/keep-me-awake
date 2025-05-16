// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

import GLib from "gi://GLib?version=2.0";

import { setConsoleLogDomain } from "console";
import { pgettext as C_ } from "gettext";

import { KMAApplication } from "./app.js";
import * as config from "./config.js";

export const main = (argv: string[]) => {
  setConsoleLogDomain(config.G_LOG_DOMAIN);

  console.debug("Running in flatpak?: ", config.runningInFlatpak());

  GLib.set_application_name(C_("application-name", "Keep Me Awake"));

  config.loadAndRegisterResources();

  const app = new KMAApplication();
  app.set_version(config.VERSION);
  app.run(argv);
};
