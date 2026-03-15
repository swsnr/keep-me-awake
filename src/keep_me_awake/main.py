# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

"""Entry point for Keep me Awake."""

import os
import sys
from gettext import pgettext as C_
from importlib.metadata import distribution
from pathlib import Path
from typing import Never

import gi
import gi.events  # pyright: ignore[reportMissingImports]

from keep_me_awake import __version__

gi.disable_legacy_autoinit()
gi.require_version("Adw", "1")
gi.require_version("Xdp", "1.0")


def is_editable_installation() -> bool:
    """Whether the application is installed in editable mode for development."""
    dist = distribution("keep_me_awake")

    if dist.origin and hasattr(dist.origin, "dir_info"):
        return getattr(dist.origin.dir_info, "editable", False)  # pyright: ignore[reportAny]

    return False


def main() -> Never:
    """Start the application, as main entry point.

    Setup environment and start the application.
    """
    from gi.repository import GLib

    from . import log

    if is_editable_installation():
        log.message("Editable installation, setting up resource overlays")
        overlays = os.environ.get("G_RESOURCE_OVERLAYS", "")
        resources_dir = Path(__file__).parent / "resources"
        overlay = f"/de/swsnr/keepmeawake={resources_dir}"
        os.environ["G_RESOURCE_OVERLAYS"] = (
            f"{overlay}:{overlays}" if overlays else overlay
        )

    # TODO: Setup translations
    # TODO: Register resources

    GLib.set_application_name(C_("application-name", "Keep me Awake"))

    # Import app only after we've set up resource overrides, etc. to make
    # sure that templates, translations, etc. are in place.
    from .app import KeepMeAwakeApplication

    # TODO: Read application ID from some file written at installation time
    app = KeepMeAwakeApplication("de.swsnr.keepmeawake.Devel")
    app.set_version(__version__)
    with gi.events.GLibEventLoopPolicy():  # pyright: ignore[reportUnknownMemberType]
        sys.exit(app.run(sys.argv))
