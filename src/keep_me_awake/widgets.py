# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

"""Widgets used by the application."""

from gi.repository import Adw, GObject, Gtk

from .enums import Inhibit


@Gtk.Template(resource_path="/de/swsnr/keepmeawake/application-window.ui")
class KeepMeAwakeApplicationWindow(Adw.ApplicationWindow):
    """The main application window."""

    __gtype_name__: str = "KeepMeAwakeApplicationWindow"

    @GObject.Property(type=bool, default=False)
    def show_update_indicator(self) -> bool:
        """Whether to show an indicator for an available update."""
        return False

    @Gtk.Template.Callback()  # pyright: ignore[reportAny]
    @staticmethod
    def icon_name(_window: KeepMeAwakeApplicationWindow, inhibit: Inhibit) -> str:
        """Determine the name of the icon to use for `inhibit`."""
        match inhibit:
            case Inhibit.NOTHING:
                return "cup-empty"
            case Inhibit.SUSPEND:
                return "cup-full"
            case Inhibit.SUSPEND_AND_IDLE:
                return "cup-full-steaming"
