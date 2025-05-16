#!@GJS@ -m
// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/**
 * Entrypoint for installed builds.
 */

import System from "system";

import { main } from "file:///@PREFIX@share/@APPID@/js/main.js";

main([System.programInvocationName].concat(ARGV));
