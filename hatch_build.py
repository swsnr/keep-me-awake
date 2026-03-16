# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12


"""Custom build plugins for hatch."""

import os
from functools import cached_property
from pathlib import Path
from subprocess import run
from typing import Any, cast, override

from hatchling.builders.config import BuilderConfig
from hatchling.builders.hooks.plugin.interface import BuildHookInterface
from packaging.version import Version


class CustomBuildHook(BuildHookInterface[BuilderConfig]):
    """Custom build hook for Keep me Awake.

    Handles translations and builds various files required for Gnome apps.
    """

    @cached_property
    def app_id(self) -> str:
        """Derive the application ID from the package version."""
        version = Version(self.metadata.version)  # pyright: ignore[reportUnknownMemberType]
        if version.is_devrelease:
            return "de.swsnr.keepmeawake.Devel"
        else:
            return "de.swsnr.keepmeawake"

    def _patch_app_id(self, source: Path, dest: Path | None = None) -> Path:
        contents = source.read_text()
        if not dest:
            dest = Path(self.build_config.directory) / source.name
        dest.write_text(contents.replace("de.swsnr.keepmeawake", self.app_id))
        return dest

    @override
    def initialize(self, version: str, build_data: dict[str, Any]) -> None:
        super().initialize(version, build_data)

        root = Path(self.root)
        for package in self.build_config.packages:
            resources_directory = root / package / "resources"
            blueprints = list(resources_directory.glob("**/*.blp"))
            n_blueprints = len(blueprints)
            self.app.display_info(
                f"Building {n_blueprints} blueprints in package {package}"
            )
            _ = run(
                [
                    "blueprint-compiler",
                    "batch-compile",
                    str(resources_directory),
                    str(resources_directory),
                ]
                + [str(p) for p in blueprints],
                check=True,
            )

            # TODO: Translate metadata file instead of copying it
            self._patch_app_id(
                root / "de.swsnr.keepmeawake.metainfo.xml",
                resources_directory / "metainfo.xml",
            )

        # Generate shared data files
        self.app.display_info("Generating shared data files")

        if self.target_name == "wheel":
            # When building a wheel build a binary resource file for Gio
            for package in self.build_config.packages:
                resources_directory = root / package / "resources"
                compiled_resources = root / package / "resources.gresource"

                self.app.display_info("Compiling GLib resources")
                run(
                    [
                        "glib-compile-resources",
                        f"--sourcedir={resources_directory}",
                        f"--target={compiled_resources}",
                        resources_directory / "resources.gresource.xml",
                    ],
                    check=True,
                )

            shared_data = cast(dict[str, str], build_data["shared_data"])
            desktop_file = self._patch_app_id(root / "de.swsnr.keepmeawake.desktop")
            shared_data[str(desktop_file)] = f"share/applications/{self.app_id}.desktop"
            for package in self.build_config.packages:
                resources_directory = root / package / "resources"
                shared_data[str(resources_directory / "metainfo.xml")] = (
                    f"share/metainfo/{self.app_id}.metainfo.xml"
                )
