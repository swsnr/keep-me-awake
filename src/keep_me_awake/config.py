# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

from importlib.metadata import distribution

# TODO: Read app ID from resource file
APP_ID: str = "de.swsnr.keepmeawake.Devel"


def is_editable_installation() -> bool:
    """Whether the application is installed in editable mode for development."""
    dist = distribution("keep_me_awake")

    if dist.origin and hasattr(dist.origin, "dir_info"):
        return getattr(dist.origin.dir_info, "editable", False)  # pyright: ignore[reportAny]

    return False
