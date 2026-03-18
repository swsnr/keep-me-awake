# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

"""The main Keep me Awake application."""

import asyncio
from dataclasses import dataclass
from gettext import gettext as _
from gettext import pgettext as C_
from typing import cast, override

from gi.repository import Adw, Gio, GLib, GObject, Gtk, Xdp, XdpGtk4

import keep_me_awake

from . import log
from .enums import Inhibit
from .widgets import KeepMeAwakeApplicationWindow


@dataclass
class _InhibitState:
    inhibit_idle: bool
    cookie: int


class FlatpakAppUpdatedMonitor(GObject.Object):
    """Monitor for updates of the installed flatpak app."""

    def __init__(self) -> None:
        """Create a new monitor for app updates."""
        super().__init__()
        self._was_updated: bool = False
        app_dir = Gio.File.new_for_path("/app")
        self._monitor: Gio.FileMonitor | None = None
        if app_dir.query_exists():
            self._monitor = app_dir.get_child(".updated").monitor(
                Gio.FileMonitorFlags.NONE, None
            )
            self._monitor.connect("changed", self._handle_change)

    def _handle_change(
        self,
        _monitor: Gio.FileMonitor,
        _file: Gio.File,
        _other_file: Gio.File | None,
        event_type: Gio.FileMonitorEvent,
    ) -> None:
        match event_type:
            case Gio.FileMonitorEvent.CREATED:
                self._was_updated = True
                self.notify("was-updated")
            case Gio.FileMonitorEvent.DELETED:
                self._was_updated = False
                self.notify("was-updated")
            case _:
                pass

    @GObject.Property(type=bool, default=False)
    def was_updated(self) -> bool:
        """Whether the app was updated and needs to be restarted."""
        return self._was_updated


class KeepMeAwakeApplication(Adw.Application):
    """Keep me Awake application class.

    The application handles inhibit state and provides the main window.
    """

    __gtype_name__: str = "KeepMeAwakeApplication"

    def __init__(self, application_id: str) -> None:
        """Create a new application with the given `application_id`.

        Set application ID, configure the base path for resources of this
        application, and disable inhibition initially.
        """
        super().__init__(
            application_id=application_id, resource_base_path="/de/swsnr/keepmeawake"
        )
        self._inhibit_state: _InhibitState | None = None
        self._background_task: asyncio.Task[None] | None = None
        self._portal: Xdp.Portal = Xdp.Portal.new()
        if self._portal.running_under_flatpak():
            self._app_updated_monitor = FlatpakAppUpdatedMonitor()

    def _update_notification(self) -> None:
        """Update the notification to indicate current inhibition state."""
        notification = None
        if self._inhibit_state:
            if self._inhibit_state.inhibit_idle:
                title = C_(
                    "notification.title",
                    "Suspend and screen lock inhibited",
                )
                body = C_(
                    "notification.body",
                    "Keep me Awake inhibits suspend and screen lock at your request.",
                )
            else:
                title = C_(
                    "notification.title",
                    "Suspend inhibited",
                )
                body = C_(
                    "notification.body",
                    "Keep me Awake inhibits suspend at your request.",
                )
            notification = Gio.Notification.new(title)
            notification.set_body(body)

        app_id = self.get_application_id()
        assert app_id is not None
        notification_id = f"{app_id}.persistent-inhibitor-notification"
        if notification:
            self.send_notification(notification_id, notification)
        else:
            self.withdraw_notification(notification_id)

    def _activate_about(
        self, _act: Gio.SimpleAction, _parameter: GLib.Variant | None = None
    ) -> None:
        version = self.get_version()
        assert version is not None
        (major, minor) = version.split(".")[:2]
        dialog = Adw.AboutDialog.new_from_appdata(
            "/de/swsnr/keepmeawake/metainfo.xml", f"{major}.{minor}.0"
        )
        dialog.set_version(version)
        dialog.set_license_type(Gtk.License.CUSTOM)
        dialog.set_license(
            C_(
                "about-dialog.license-text",
                # Translators: This is Pango markup, be sure to escape appropriately
                """Copyright {copyright_name} &lt;{copyright_email}&gt;

Licensed under the terms of the EUPL 1.2. You can find official translations
of the license text at <a href=\"{translations}\">{translations}</a>.

The full English text follows.

{license_text}""",
            ).format(
                copyright_name="Sebastian Wiesner",
                copyright_email="sebastian@swsnr.de",
                translations="https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12",
                license_text=keep_me_awake.license_text(),
            )
        )
        dialog.add_link(
            C_("about-dialog.link.label", "Translations"),
            "https://translate.codeberg.org/engage/de-swsnr-keepmeawake/",
        )

        dialog.set_developers(["Sebastian Wiesner https://swsnr.de"])
        dialog.set_designers(["Sebastian Wiesner https://swsnr.de"])
        # Credits for the translator to the current language.
        # Translators: Add your name here, as "Jane Doe <jdoe@example.com>" or
        # "Jane Doe https://jdoe.example.com". Mail address or URL are optional.
        # Separate multiple translators with a newline, i.e. \n
        dialog.set_translator_credits(_("translator-credits"))
        dialog.add_acknowledgement_section(
            C_(
                "about-dialog.acknowledgment-section",
                "Helpful services",
            ),
            [
                "Codeberg https://codeberg.org",
                "Flathub https://flathub.org/",
                "Open Build Service https://build.opensuse.org/",
            ],
        )
        dialog.add_other_app(
            "de.swsnr.pictureoftheday",
            # Translators: Use app name from https://flathub.org/apps/de.swsnr.pictureoftheday
            C_("about-dialog.other-app.name", "Picture Of The Day"),
            C_(
                "about-dialog.other-app.summary",
                # Translators: Use summary from https://flathub.org/apps/de.swsnr.pictureoftheday
                "Your daily wallpaper",
            ),
        )
        dialog.add_other_app(
            "de.swsnr.turnon",
            # Translators: Use app name from https://flathub.org/apps/de.swsnr.turnon
            C_("about-dialog.other-app.name", "Turn On"),
            C_(
                "about-dialog.other-app.summary",
                # Translators: Use summary from https://flathub.org/apps/de.swsnr.turnon
                "Turn on devices in your network",
            ),
        )
        dialog.present(self.get_active_window())

    def _activate_quit(
        self, _act: Gio.SimpleAction, _parameter: GLib.Variant | None = None
    ) -> None:
        self.inhibitors = Inhibit.NOTHING
        window = self.get_active_window()
        if window:
            window.close()

    def _activate_toggle_inhibit(
        self, _act: Gio.SimpleAction, _parameter: GLib.Variant | None = None
    ) -> None:
        self.inhibitors = (
            Inhibit.SUSPEND_AND_IDLE
            if self.inhibitors == Inhibit.NOTHING
            else Inhibit.NOTHING
        )

    def _setup_actions(self) -> None:
        quit = Gio.SimpleAction(name="quit")
        _ = quit.connect("activate", self._activate_quit)
        about = Gio.SimpleAction(name="about")
        _ = about.connect("activate", self._activate_about)
        toggle_inhibit = Gio.SimpleAction(name="toggle-inhibit")
        toggle_inhibit.connect("activate", self._activate_toggle_inhibit)

        actions = [
            quit,
            about,
            toggle_inhibit,
            Gio.PropertyAction(name="inhibit", object=self, property_name="inhibitors"),
        ]
        for action in actions:
            self.add_action(action)

        self.set_accels_for_action("app.quit", ["<Control>q"])

    def _main_window_close_request(self, _window: Gtk.Window) -> bool:
        """Handle close requests for the main window.

        Update the notification to remind the user that they might still be
        inhibiting idle or suspend.
        """
        self._update_notification()
        # Allow the close request to proceed
        return False

    async def _ask_background(self) -> None:
        # TODO: async not typed correctly: https://github.com/pygobject/pygobject-stubs/issues/220
        parent = None
        window = self.get_active_window()
        if window:
            parent = XdpGtk4.parent_new_gtk(window)
        success = await self._portal.request_background(  # pyright: ignore[reportUnknownVariableType, reportGeneralTypeIssues]
            parent,
            C_(
                "portal.request-background.reason",
                "Inhibit suspend and idle without a main window",
            ),
            None,  # pyright: ignore[reportArgumentType]
            Xdp.BackgroundFlags.NONE,
        )
        if not cast(bool, success):
            log.warn("Failed to request background permission!")
            pass

    @GObject.Property(type=Inhibit, default=Inhibit.NOTHING)
    def inhibitors(self) -> Inhibit:
        """Get the current inhibitors."""
        if self._inhibit_state:
            return (
                Inhibit.SUSPEND_AND_IDLE
                if self._inhibit_state.inhibit_idle
                else Inhibit.SUSPEND
            )
        else:
            return Inhibit.NOTHING

    @inhibitors.setter
    def set_inhibitors(self, inhibit: Inhibit) -> None:
        """Update inhibitors.

        Disable current inhibitors, if any, then inhibit according to `inhibit`
        and emit the notify signal for this property.
        """
        if self._inhibit_state:
            self.uninhibit(self._inhibit_state.cookie)
            self._inhibit_state = None
        match inhibit:
            case Inhibit.NOTHING:
                # Release hold when we have nothing to inhibit
                self.release()
            case other:
                flags: Gtk.ApplicationInhibitFlags = Gtk.ApplicationInhibitFlags.SUSPEND
                if other == Inhibit.SUSPEND_AND_IDLE:
                    flags |= Gtk.ApplicationInhibitFlags.IDLE
                # Keep app alive while inhibiting
                self.hold()
                cookie = self.inhibit(
                    None,
                    flags,
                    C_(
                        "inhibit-reason",
                        "Keep me Awake inhibits suspend at your request.",
                    ),
                )
                self._inhibit_state = _InhibitState(
                    inhibit_idle=Gtk.ApplicationInhibitFlags.IDLE in flags,
                    cookie=cookie,
                )
        self._update_notification()

    @override
    def do_activate(self) -> None:
        Adw.Application.do_activate(self)

        window = self.get_active_window()
        if not window:
            window = KeepMeAwakeApplicationWindow(application=self)
            window.connect("close-request", self._main_window_close_request)

            app_id = self.get_application_id()
            if app_id and app_id.endswith(".Devel"):
                window.add_css_class("devel")

            if self._app_updated_monitor:
                self._app_updated_monitor.bind_property(
                    "was-updated",
                    window,
                    "show-update-indicator",
                    GObject.BindingFlags.SYNC_CREATE,
                )

        window.present()

        self.activate_action("inhibit", GLib.Variant.new_string("suspend-and-idle"))

        if self._portal.running_under_sandbox() and not self._background_task:
            # Request background but only if not already requested
            self._background_task = asyncio.create_task(self._ask_background())

    @override
    def do_startup(self) -> None:
        Adw.Application.do_startup(self)

        app_id = self.get_application_id()
        if app_id:
            Gtk.Window.set_default_icon_name(app_id)

        self._setup_actions()
