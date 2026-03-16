# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

"""Entry point for Keep me Awake."""

import os
import sys
from gettext import pgettext as C_
from importlib import resources
from pathlib import Path
from typing import Never

import gi
import gi.events  # pyright: ignore[reportMissingImports]
from gi.repository import Gio  # pyright: ignore[reportMissingImports]
from packaging.version import Version

import keep_me_awake

gi.disable_legacy_autoinit()
gi.require_version("Adw", "1")
gi.require_version("Xdp", "1.0")


def main() -> Never:
    """Start the application, as main entry point.

    Setup environment and start the application.
    """
    from gi.repository import GLib

    from . import log

    # Default to a devel build
    app_id: str
    if keep_me_awake.is_installed_editable():
        log.message("Editable installation, setting up resource overlays")
        overlays = os.environ.get("G_RESOURCE_OVERLAYS", "")
        resources_dir = Path(__file__).parent / "resources"
        overlay = f"/de/swsnr/keepmeawake={resources_dir}"
        os.environ["G_RESOURCE_OVERLAYS"] = (
            f"{overlay}:{overlays}" if overlays else overlay
        )
        app_id = "de.swsnr.keepmeawake.Devel"
    else:
        # Read compiled resources
        with resources.as_file(
            keep_me_awake.resource_files() / "resources.gresource"
        ) as resource:
            log.message(f"Loading compiled resources from {resource}")
            Gio.resources_register(Gio.Resource.load(str(resource)))
        version = Version(keep_me_awake.version())
        if version.is_devrelease:
            app_id = "de.swsnr.keepmeawake.Devel"
        else:
            app_id = "de.swsnr.keepmeawake"

    # TODO: Setup translations
    GLib.set_application_name(C_("application-name", "Keep me Awake"))

    # Import app only after we've set up resource overrides, etc. to make
    # sure that templates, translations, etc. are in place.
    from .app import KeepMeAwakeApplication

    app = KeepMeAwakeApplication(application_id=app_id)
    app.set_version(keep_me_awake.version())
    with gi.events.GLibEventLoopPolicy():  # pyright: ignore[reportUnknownMemberType]
        sys.exit(app.run(sys.argv))
