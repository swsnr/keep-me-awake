# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12

from pathlib import Path
import sys
import os
from typing import Never

from .config import is_editable_installation


def setup_environment() -> None:
    if is_editable_installation():
        overlays = os.environ.get("G_RESOURCE_OVERLAYS", "")
        resources_dir = Path(__file__).parent / "resources"
        overlay = f"/de/swsnr/keepmeawake={resources_dir}"
        os.environ["G_RESOURCE_OVERLAYS"] = (
            f"{overlay}:{overlays}" if overlays else overlay
        )


def main() -> Never:
    setup_environment()

    # Import app only after we've set up resource overrides, etc. to make
    # sure that templates, translations, etc. are in place.
    from .app import KeepMeAwakeApplication

    app = KeepMeAwakeApplication()
    sys.exit(app.run(sys.argv))
