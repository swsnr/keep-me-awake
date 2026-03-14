# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12


from .gi import GObject


class Inhibit(GObject.GEnum):
    NOTHING = 1
    SUSPEND = 2
    SUSPEND_AND_IDLE = 3
