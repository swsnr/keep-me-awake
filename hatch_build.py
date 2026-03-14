# Copyright Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the EUPL
#
# See https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12


from pathlib import Path
from subprocess import run
from typing import Any, override
from hatchling.builders.config import BuilderConfig
from hatchling.builders.hooks.plugin.interface import BuildHookInterface


class CustomBuildHook(BuildHookInterface[BuilderConfig]):
    @override
    def finalize(
        self,
        version: str,
        build_data: dict[str, Any],  # pyright: ignore[reportExplicitAny]
        artifact_path: str,
    ) -> None:
        super().finalize(version, build_data, artifact_path)

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
