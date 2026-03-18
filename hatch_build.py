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

    def _build_blueprint(self, package: str) -> None:
        resources_directory = Path(self.root) / package / "resources"
        blueprints = list(resources_directory.glob("**/*.blp"))
        n_blueprints = len(blueprints)
        self.app.display_info(
            f"Building {n_blueprints} blueprints in package {package}"
        )
        try:
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
        except FileNotFoundError as error:
            self.app.display_error(f"blueprint-compiler missing: {error}")

    def _translate_metainfo(self, package: str) -> None:
        root = Path(self.root)
        metainfo_file = root / package / "resources" / "metainfo.xml"
        self.app.display_info("Translating metainfo file")
        try:
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
        except FileNotFoundError as error:
            self.app.display_error(f"msgfmt missing: {error}")

    def _compile_resources(self, package: str) -> None:
        root = Path(self.root)
        resources_directory = root / package / "resources"
        compiled_resources = root / package / "resources.gresource"
        self.app.display_info("Compiling GLib resources")
        try:
            _ = run(
                [
                    "glib-compile-resources",
                    f"--sourcedir={resources_directory}",
                    f"--target={compiled_resources}",
                    resources_directory / "resources.gresource.xml",
                ],
                check=True,
            )
        except FileNotFoundError as error:
            self.app.display_error(f"glib-compile-resources missing: {error}")

    def _compile_message_catalogs(self, shared_data: dict[str, str]) -> None:
        self.app.display_info("Compiling message catalogs")
        mo_dir = Path(self.build_config.directory) / "mo"
        mo_dir.mkdir(parents=True, exist_ok=True)
        for po_file in (Path(self.root) / "po").glob("*.po"):
            lang = po_file.stem
            mo_file = (mo_dir / lang).with_suffix(".mo")
            try:
                _ = run(["msgfmt", "-o", str(mo_file), str(po_file)], check=True)
            except FileNotFoundError as error:
                self.app.display_error(f"msgfmt missing: {error}")
                return
            else:
                shared_data[str(mo_file)] = (
                    f"share/locale/{lang}/LC_MESSAGES/{self.app_id}.mo"
                )

    def _translate_desktop_file(self, shared_data: dict[str, str]) -> None:
        self.app.display_info("Translating desktop file")
        root = Path(self.root)
        desktop_file = (
            Path(self.build_config.directory) / "de.swsnr.keepmeawake.desktop"
        )
        try:
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
        except FileNotFoundError as error:
            self.app.display_error(f"msgfmt missing: {error}")
            return
        else:
            self._patch_app_id(desktop_file)
            shared_data[str(desktop_file)] = f"share/applications/{self.app_id}.desktop"

    @override
    def initialize(self, version: str, build_data: dict[str, Any]) -> None:  # pyright: ignore[reportExplicitAny]
        super().initialize(version, build_data)

        root = Path(self.root)
        shared_data = cast(dict[str, str], build_data["shared_data"])

        for package in self.build_config.packages:
            self._build_blueprint(package)
            self._translate_metainfo(package)

        if self.target_name == "wheel":
            # When building a wheel build a binary resource file for Gio
            for package in self.build_config.packages:
                self._compile_resources(package)

                resources_directory = root / package / "resources"

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

            self._compile_message_catalogs(shared_data)
            self._translate_desktop_file(shared_data)

            self.app.display_info("Copying D-Bus service")
            service = root / "dbus-1" / "de.swsnr.keepmeawake.service"
            dest = service.copy(Path(self.build_config.directory) / service.name)
            self._patch_app_id(dest)
            shared_data[str(dest)] = f"share/dbus-1/services/{self.app_id}.service"
