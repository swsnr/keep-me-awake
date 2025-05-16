// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

import GObject from "gi://GObject?version=2.0";
import Adw from "gi://Adw?version=1";

export const KMAApplicationWindow = GObject.registerClass(
  {
    GTypeName: "KMAApplicationWindow",
    Template: "resource:///de/swsnr/keepmeawake/ui/application-window.ui",
  },
  class extends Adw.ApplicationWindow {},
);

export default KMAApplicationWindow;
