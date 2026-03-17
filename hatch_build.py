# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12


"""Custom build plugins for hatch."""

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

    def _patch_app_id(self, source: Path) -> None:
        contents = source.read_text()
        _ = source.write_text(contents.replace("de.swsnr.keepmeawake", self.app_id))

    @override
    def initialize(self, version: str, build_data: dict[str, Any]) -> None:  # pyright: ignore[reportExplicitAny]
        super().initialize(version, build_data)

        root = Path(self.root)
        shared_data = cast(dict[str, str], build_data["shared_data"])

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

            metainfo_file = resources_directory / "metainfo.xml"
            self.app.display_info("Translating metainfo file")
            _ = run(
                [
                    "msgfmt",
                    "--xml",
                    "--template",
                    str(root / "de.swsnr.keepmeawake.metainfo.xml"),
                    "-d",
                    str(root / "po"),
                    "--output",
                    str(metainfo_file),
                ],
                check=True,
            )
            self._patch_app_id(metainfo_file)

        if self.target_name == "wheel":
            # When building a wheel build a binary resource file for Gio
            for package in self.build_config.packages:
                resources_directory = root / package / "resources"
                compiled_resources = root / package / "resources.gresource"

                self.app.display_info("Compiling GLib resources")
                _ = run(
                    [
                        "glib-compile-resources",
                        f"--sourcedir={resources_directory}",
                        f"--target={compiled_resources}",
                        resources_directory / "resources.gresource.xml",
                    ],
                    check=True,
                )

                self.app.display_info("Copying translated metadata file")
                shared_data[str(resources_directory / "metainfo.xml")] = (
                    f"share/metainfo/{self.app_id}.metainfo.xml"
                )

                self.app.display_info("Copying icons")
                app_icon = (
                    resources_directory
                    / "icons"
                    / "scalable"
                    / "apps"
                    / f"{self.app_id}.svg"
                )
                shared_data[str(app_icon)] = (
                    f"share/icons/hicolor/scalable/apps/{self.app_id}.svg"
                )
                symbolic_icon = (
                    resources_directory
                    / "icons"
                    / "symbolic"
                    / "apps"
                    / "de.swsnr.keepmeawake-symbolic.svg"
                )
                shared_data[str(symbolic_icon)] = (
                    f"share/icons/hicolor/symbolic/apps/{self.app_id}-symbolic.svg"
                )

            self.app.display_info("Compiling message catalogs")
            mo_dir = Path(self.build_config.directory) / "mo"
            mo_dir.mkdir(parents=True, exist_ok=True)
            for po_file in (root / "po").glob("*.po"):
                lang = po_file.stem
                mo_file = (mo_dir / lang).with_suffix(".mo")
                _ = run(["msgfmt", "-o", str(mo_file), str(po_file)], check=True)
                shared_data[str(mo_file)] = (
                    f"share/locale/{lang}/LC_MESSAGES/{self.app_id}.mo"
                )

            self.app.display_info("Translating desktop file")
            desktop_file = (
                Path(self.build_config.directory) / "de.swsnr.keepmeawake.desktop"
            )
            _ = run(
                [
                    "msgfmt",
                    "--desktop",
                    "--template",
                    str(root / "de.swsnr.keepmeawake.desktop"),
                    "-d",
                    str(root / "po"),
                    "--output",
                    str(desktop_file),
                ],
                check=True,
            )
            self._patch_app_id(desktop_file)
            shared_data[str(desktop_file)] = f"share/applications/{self.app_id}.desktop"

            self.app.display_info("Copying D-Bus service")
            service = root / "dbus-1" / "de.swsnr.keepmeawake.service"
            dest = service.copy(Path(self.build_config.directory) / service.name)
            self._patch_app_id(dest)
            shared_data[str(dest)] = f"share/dbus-1/services/{self.app_id}.service"
