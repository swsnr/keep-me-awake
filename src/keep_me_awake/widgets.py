# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

"""Widgets used by the application."""

from typing import Any

from gi.repository import Adw, GObject, Gtk

from .enums import Inhibit


@Gtk.Template(resource_path="/de/swsnr/keepmeawake/application-window.ui")
class KeepMeAwakeApplicationWindow(Adw.ApplicationWindow):
    """The main application window."""

    __gtype_name__: str = "KeepMeAwakeApplicationWindow"

    def __init__(self, application: Adw.Application) -> None:
        """Create a new main window instance."""
        super().__init__(application=application)
        self._show_update_indicator: bool = False

    @GObject.Property(type=bool, default=False)
    def show_update_indicator(self) -> bool:
        """Whether to show an indicator for an available update."""
        return self._show_update_indicator

    @show_update_indicator.setter
    def set_show_update_indicator(self, value: bool) -> None:
        """Change update indicator visibility."""
        self._show_update_indicator = value

    @Gtk.Template.Callback()
    @staticmethod
    def icon_name(_window: Any, inhibit: Inhibit) -> str:  # noqa: ANN401
        """Determine the name of the icon to use for `inhibit`."""
        match inhibit:
            case Inhibit.NOTHING:
                return "cup-empty"
            case Inhibit.SUSPEND:
                return "cup-full"
            case Inhibit.SUSPEND_AND_IDLE:
                return "cup-full-steaming"
