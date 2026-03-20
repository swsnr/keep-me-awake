# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

"""Entry point for Keep me Awake."""

import gettext
import locale
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
gi.require_version("XdpGtk4", "1.0")


def main() -> Never:
    """Start the application, as main entry point.

    Setup environment and start the application.
    """
    from gi.repository import GLib, Xdp

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
            log.info(f"Loading compiled resources from {resource}")
            Gio.resources_register(Gio.Resource.load(str(resource)))
        version = Version(keep_me_awake.version())
        if version.is_devrelease:
            app_id = "de.swsnr.keepmeawake.Devel"
        else:
            app_id = "de.swsnr.keepmeawake"

    prefix = Path("/app") if Xdp.Portal.running_under_flatpak() else Path(sys.prefix)
    locale_dir = prefix / "share" / "locale"
    log.info(f"Loading translations from {locale_dir}")

    locale.setlocale(locale.LC_ALL, "")
    # Setup text domain for the C standard library, and by implication for glib,
    # which exposes translations to Gtk Builder and thus to blueprint.
    locale.bindtextdomain(app_id, locale_dir)
    locale.bind_textdomain_codeset(app_id, "UTF-8")
    locale.textdomain(app_id)
    # Setup text domain for Python's gettext, so that our messages in Python code
    # get translated.
    gettext.bindtextdomain(app_id, locale_dir)
    gettext.textdomain(app_id)

    GLib.set_application_name(C_("application-name", "Keep me Awake"))

    # Import app only after we've set up resource overrides, etc. to make
    # sure that templates, translations, etc. are in place.
    from .app import KeepMeAwakeApplication

    app = KeepMeAwakeApplication(application_id=app_id)
    app.set_version(keep_me_awake.version())

    # Use GLib event policy as a context manager instead of setting the policy
    # explicitly as recommended in https://pygobject.gnome.org/guide/asynchronous.html
    #
    # This maintains forward compatibility with https://gitlab.gnome.org/GNOME/pygobject/-/merge_requests/503
    # in PyGObject 3.56, and the upcoming deprecation of event loop policies in
    # Python 3.14
    #
    # We also have to ignore typing here because for some reason pyright doesn't find
    # the gi.events module.
    with gi.events.GLibEventLoopPolicy():  # pyright: ignore[reportUnknownMemberType]
        sys.exit(app.run(sys.argv))
