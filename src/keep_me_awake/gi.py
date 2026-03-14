# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

"""
Central GI import for this application.

Setup GI properly, and import required libraries.
"""

import gi

gi.disable_legacy_autoinit()

gi.require_version("Adw", "1")

from gi.repository import Adw, Gtk, GLib, Gio, GObject  # pyright: ignore[reportMissingModuleSource, reportUnusedImport]  # noqa: E402, F401
