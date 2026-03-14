# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

from .gi import Adw, Gtk, GObject
from .inhibit import Inhibit


@Gtk.Template(resource_path="/de/swsnr/keepmeawake/ui/application-window.ui")
class KeepMeAwakeApplicationWindow(Adw.ApplicationWindow):
    __gtype_name__: str = "KeepMeAwakeApplicationWindow"

    @GObject.Property(type=bool, default=False)
    def show_update_indicator(self) -> bool:
        return False

    @Gtk.Template.Callback()  # pyright: ignore[reportAny]
    @staticmethod
    def icon_name(_window: KeepMeAwakeApplicationWindow, inhibit: Inhibit) -> str:
        match inhibit:
            case Inhibit.NOTHING:
                return "cup-empty"
            case Inhibit.SUSPEND:
                return "cup-full"
            case Inhibit.SUSPEND_AND_IDLE:
                return "cup-full-steaming"
