// @ts-check

import eslint from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  eslint.configs.recommended,
  tseslint.configs.strictTypeChecked,
  tseslint.configs.stylisticTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: {
          allowDefaultProject: ["run.js", "eslint.config.mjs"],
        },
        // @ts-expect-error Missing types for nodejs
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    ignores: [
      ".flatpak-builddir/",
      ".flatpak-builder/",
      ".flatpak-repo/",
      "build/",
      "run.js",
      "entrypoint.js",
      "node_modules/",
    ],
  },
);
