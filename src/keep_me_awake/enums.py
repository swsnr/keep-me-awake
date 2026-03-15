# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12


"""Shared enumerations."""

from gi.repository import GObject


class Inhibit(GObject.GEnum):
    """Different modes for inhibiting idle and suspend."""

    NOTHING = 1
    SUSPEND = 2
    SUSPEND_AND_IDLE = 3
