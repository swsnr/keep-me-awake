# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12


"""Custom build plugins for hatch."""

import os
from collections.abc import Mapping
from pathlib import Path
from shutil import copy
from subprocess import run
from typing import Any, cast, override

from hatchling.builders.config import BuilderConfig
from hatchling.builders.hooks.plugin.interface import BuildHookInterface
from packaging.version import Version


def drop_virtualenv(env: Mapping[str, str]) -> Mapping[str, str]:
    """Drop current virtualenv from environment.

    If the given environment contains `$VIRTUAL_ENV`, return a new mapping
    without `$VIRTUAL_ENV` and with an updated `$PATH` where any entries
    pointing to that virtualenv have been removed.

    Otherwise return `env` itself.
    """
    venv = env.get("VIRTUAL_ENV")
    if venv:
        env = dict(env)
        del env["VIRTUAL_ENV"]
        paths = env["PATH"].split(os.pathsep)
        env["PATH"] = os.pathsep.join(
            p
            for p in paths
            if not Path(p).exists() or not Path(p).samefile(Path(venv) / "bin")
        )
        return env
    else:
        return env


class CustomBuildHook(BuildHookInterface[BuilderConfig]):
    """Custom build hook for Keep me Awake.

    Handles translations and builds various files required for Gnome apps.
    """

    def _patch_app_id(self, source: Path, app_id: str) -> None:
        contents = source.read_text()
        _ = source.write_text(contents.replace("de.swsnr.keepmeawake", app_id))

    @override
    def initialize(self, version: str, build_data: dict[str, Any]) -> None:
        super().initialize(version, build_data)

        if self.target_name != "wheel":
            return

        app_id = "de.swsnr.keepmeawake.Devel"
        if version != "editable" and not Version(self.metadata.version).is_devrelease:  # pyright: ignore[reportUnknownMemberType]
            app_id = "de.swsnr.keepmeawake"

        root = Path(self.root)
        resources_directory = root / "resources"

        shared_data = cast(dict[str, str], build_data["shared_data"])

        output_directory = Path(self.build_config.directory)
        resources_out_directory = output_directory / "resources"
        resources_out_directory.mkdir(parents=True, exist_ok=True)

        if os.environ.get("SKIP_BLUEPRINT") != "1":
            blueprints = list(resources_directory.glob("**/*.blp"))
            self.app.display_info("Compiling blueprint files")
            _ = run(
                [
                    "blueprint-compiler",
                    "batch-compile",
                    str(resources_out_directory),
                    str(resources_directory),
                ]
                + [str(p) for p in blueprints],
                check=True,
                # Blueprint needs to run against whatever Python it was installed to,
                # so drop the virtualenv from its environment
                env=drop_virtualenv(os.environ),
            )

        metainfo_file = resources_out_directory / "metainfo.xml"
        if os.environ.get("SKIP_MSGFMT") != "1":
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
        else:
            copy(root / "de.swsnr.keepmeawake.metainfo.xml", metainfo_file)
        self._patch_app_id(metainfo_file, app_id)

        if os.environ.get("SKIP_MSGFMT") != "1":
            self.app.display_info("Compiling message catalogs to share/locale")
            locale_dir = Path(self.build_config.directory) / "locale"
            for po_file in (Path(self.root) / "po").glob("*.po"):
                lang = po_file.stem
                mo_file = locale_dir / lang / "LC_MESSAGES" / f"{app_id}.mo"
                mo_file.parent.mkdir(parents=True, exist_ok=True)
                _ = run(["msgfmt", "-o", str(mo_file), str(po_file)], check=True)
                shared_data[str(mo_file)] = (
                    f"share/locale/{lang}/LC_MESSAGES/{app_id}.mo"
                )

        # When installing an editable version do not compile gresources and
        # skip most of the shared data, as we don't need it in editable installs.
        if version == "editable":
            return

        if os.environ.get("SKIP_MSGFMT") != "1":
            self.app.display_info(
                "Copying translated desktop file to share/applications"
            )
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
            self._patch_app_id(desktop_file, app_id)
            shared_data[str(desktop_file)] = f"share/applications/{app_id}.desktop"

        if os.environ.get("SKIP_BLUEPRINT") != "1":
            # No blueprints, no resources
            self.app.display_info("Compiling Gio resources")
            compiled_resources = output_directory / "resources.gresource"
            _ = run(
                [
                    "glib-compile-resources",
                    f"--sourcedir={resources_directory}",
                    f"--sourcedir={resources_out_directory}",
                    f"--target={compiled_resources}",
                    resources_directory / "resources.gresource.xml",
                ],
                check=True,
            )
            for package in self.build_config.packages:
                build_data["force_include"][str(compiled_resources)] = (
                    f"{package}/{compiled_resources.name}"
                )

        self.app.display_info("Copying metainfo to share/metainfo")
        shared_data[str(resources_out_directory / "metainfo.xml")] = (
            f"share/metainfo/{app_id}.metainfo.xml"
        )
        self.app.display_info("Copying icons to share/icons")
        app_icon = resources_directory / "icons" / "scalable" / "apps" / f"{app_id}.svg"
        shared_data[str(app_icon)] = f"share/icons/hicolor/scalable/apps/{app_id}.svg"
        symbolic_icon = (
            resources_directory
            / "icons"
            / "symbolic"
            / "apps"
            / "de.swsnr.keepmeawake-symbolic.svg"
        )
        shared_data[str(symbolic_icon)] = (
            f"share/icons/hicolor/symbolic/apps/{app_id}-symbolic.svg"
        )
        self.app.display_info("Copying D-Bus service to share/dbus-1/services")
        service = root / "dbus-1" / "de.swsnr.keepmeawake.service"
        # TODO: Python 3.14: Use Path.copy instead
        dest = copy(service, Path(self.build_config.directory) / service.name)
        self._patch_app_id(Path(dest), app_id)
        shared_data[str(dest)] = f"share/dbus-1/services/{app_id}.service"
