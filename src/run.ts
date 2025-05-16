// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/**
 * Entrypoint for running in development.
 */

// @ts-check

import { setConsoleLogDomain } from "console";

import { G_LOG_DOMAIN } from "./config.js";
import { main } from "./main.js";

setConsoleLogDomain(G_LOG_DOMAIN);
console.info("Running from source");
main(["run.js"].concat(ARGV));
