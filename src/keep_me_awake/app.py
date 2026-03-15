# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

"""The main Keep me Awake application."""

from dataclasses import dataclass
from gettext import pgettext as C_
from typing import override

from gi.repository import (
    Adw,
    Gio,
    GLib,
    GObject,
    Gtk,
)

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

    _inhibit_state: _InhibitState | None

    def __init__(self, application_id: str) -> None:
        """Create a new application with the given `application_id`.

        Set application ID, configure the base path for resources of this
        application, and disable inhibition initially.
        """
        super().__init__(
            application_id=application_id, resource_base_path="/de/swsnr/keepmeawake"
        )
        self._inhibit_state = None

    def _activate_quit(
        self, _act: Gio.SimpleAction, _parameter: GLib.Variant | None = None
    ) -> None:
        # TODO: Clear hold guard for global shortcuts
        self.inhibitors = Inhibit.NOTHING
        window = self.get_active_window()
        if window:
            window.close()

    def _setup_actions(self) -> None:
        # TODO: configure-global-shortcuts
        # TODO: toggle-inhibit
        # TODO: shortcuts
        # TODO: About
        quit = Gio.SimpleAction(name="quit")
        _ = quit.connect("activate", self._activate_quit)
        actions = [
            quit,
            Gio.PropertyAction(name="inhibit", object=self, property_name="inhibitors"),
        ]
        for action in actions:
            self.add_action(action)

        self.set_accels_for_action("app.quit", ["<Control>q"])

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
                pass
            case other:
                flags: Gtk.ApplicationInhibitFlags = Gtk.ApplicationInhibitFlags.SUSPEND
                if other == Inhibit.SUSPEND_AND_IDLE:
                    flags |= Gtk.ApplicationInhibitFlags.IDLE
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

    @override
    def do_activate(self) -> None:
        Adw.Application.do_activate(self)

        window = self.get_active_window()
        if not window:
            window = KeepMeAwakeApplicationWindow(application=self)
        window.present()

    @override
    def do_startup(self) -> None:
        Adw.Application.do_startup(self)

        app_id = self.get_application_id()
        if app_id:
            Gtk.Window.set_default_icon_name(app_id)

        self._setup_actions()
