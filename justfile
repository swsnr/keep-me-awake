APPID := "de.swsnr.keepmeawake"
VERSION := `git describe --always`

default:
    just --list

_npm-ci:
    npm ci

# Run the typescript compiler.
compile-tsc:
    npx tsc --build
    sed -i -e 's/@APPID@/{{APPID}}/' -e 's/@VERSION@/{{VERSION}}/' build/js/config.js

# Compile blueprint resources
compile-blueprint:
    mkdir -p build/ui build/resources
    blueprint-compiler batch-compile build/ui ui ui/*.blp
    glib-compile-resources --sourcedir=build/ui \
        --target build/resources/resources.ui.gresource \
        ui/resources.ui.gresource.xml

# Build the application.
build: compile-tsc compile-blueprint

# Build for flatpak.
build-flatpak: _npm-ci build
    install -Dm0644 entrypoint.js build/entrypoint.js
    sed -i -e 's_@PREFIX@_/app/_' -e 's_@GJS@_/usr/bin/gjs_' -e 's/@APPID@/{{APPID}}/' build/entrypoint.js

# Clean build artifacts
clean:
    rm -rf build .flatpak-repo .flatpak-builddir

# Run the application
run: build
    gjs -m run.js

# Build and install development flatpak without sandboxing
flatpak-devel-install:
    flatpak run org.flatpak.Builder --force-clean --user --install \
        --install-deps-from=flathub --repo=.flatpak-repo \
        .flatpak-builddir flatpak/de.swsnr.keepmeawake.Devel.yaml

install-flatpak: build-flatpak
    mkdir -p '/app/share/{{APPID}}'
    cp -rT build/js '/app/share/{{APPID}}/js'
    install -Dm0644 -t '/app/share/{{APPID}}/resources' build/resources/*.gresource
    install -Dm0755 build/entrypoint.js '/app/bin/{{APPID}}'

format:
    npx prettier --write .
    blueprint-compiler format ui/**/*.blp --fix

lint:
    npx eslint .
    npx prettier --check .
    blueprint-compiler format ui/**/*.blp

test-all: _npm-ci build lint
