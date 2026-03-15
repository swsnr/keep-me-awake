# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

"""The main Keep me Awake application."""

import asyncio
from dataclasses import dataclass
from gettext import pgettext as C_
from typing import cast, override

from gi.repository import Adw, Gio, GLib, GObject, Gtk, Xdp

from . import log
from .enums import Inhibit
from .widgets import KeepMeAwakeApplicationWindow


@dataclass
class _InhibitState:
    inhibit_idle: bool
    cookie: int


class KeepMeAwakeApplication(Adw.Application):
    """Keep me Awake application class.

    The application provides the
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
            # TODO: Debug notification not appearing? Prob. missing desktop file?
            self.send_notification(notification_id, notification)
        else:
            self.withdraw_notification(notification_id)

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
        # TODO: About
        quit = Gio.SimpleAction(name="quit")
        _ = quit.connect("activate", self._activate_quit)
        toggle_inhibit = Gio.SimpleAction(name="toggle-inhibit")

        actions = [
            quit,
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
        success = await self._portal.request_background(  # pyright: ignore[reportUnknownVariableType, reportGeneralTypeIssues]
            None,
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
        self.notify("inhibitors")
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
