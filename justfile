APPID := "de.swsnr.keepmeawake"
VERSION := `git describe --always`

default:
    just --list

# Run the typescript compiler.
compile-tsc:
    npx tsc --build
    sed -i -e 's/@APPID@/{{APPID}}/' -e 's/@VERSION@/{{VERSION}}/' build/js/config.js

# Compile blueprint resources
compile-blueprint:
    mkdir -p build/resources-src/ui build/resources
    blueprint-compiler batch-compile build/resources-src/ui resources/ui resources/ui/*.blp

# Compile translated desktop file.
compile-desktop-file:
    mkdir -p build
    @# TODO: msgfmt instead
    install -m0644 -t build/ de.swsnr.keepmeawake.desktop
    @# Patch app ID
    sed -i '/{{APPID}}/! s/de\.swsnr\.keepmeawake/{{APPID}}/g' \
        build/de.swsnr.keepmeawake.desktop

# Compile translated metainfo file.
compile-metainfo:
    mkdir -p build/resources-src
    @# TODO: msgfmt instead
    install -m0644 -t build/ de.swsnr.keepmeawake.metainfo.xml
    @# Patch app ID
    sed -i '/{{APPID}}/! s/de\.swsnr\.keepmeawake/{{APPID}}/g' \
        build/de.swsnr.keepmeawake.metainfo.xml
    @# Copy metainfo to resource directory
    cp -t build/resources-src build/de.swsnr.keepmeawake.metainfo.xml

# Compile binary resource files.
compile-resources: compile-blueprint compile-metainfo
    glib-compile-resources --sourcedir=build/resources-src \
        --target build/resources/resources.generated.gresource \
        resources/resources.generated.gresource.xml
    glib-compile-resources --sourcedir=resources \
        --target build/resources/resources.data.gresource \
        resources/resources.data.gresource.xml

# Build the application.
build: compile-tsc compile-resources compile-desktop-file

# Build for flatpak.
build-flatpak: build
    install -Dm0644 entrypoint.js build/entrypoint.js
    sed -i -e 's_@PREFIX@_/app/_' -e 's_@GJS@_/usr/bin/gjs_' -e 's/@APPID@/{{APPID}}/' build/entrypoint.js

# Clean build artifacts
clean:
    rm -rf build .flatpak-repo .flatpak-builddir

# Run the application.
run: build
    gjs -m run.js

# Build and install development flatpak without sandboxing
flatpak-devel-install:
    flatpak run org.flatpak.Builder --force-clean --user --install \
        --install-deps-from=flathub --repo=.flatpak-repo \
        .flatpak-builddir flatpak/de.swsnr.keepmeawake.Devel.yaml

# Build (but not install) regular flatpak
flatpak-build:
    flatpak run org.flatpak.Builder --force-clean --sandbox \
        --install-deps-from=flathub --ccache --user \
        --mirror-screenshots-url=https://dl.flathub.org/media/ --repo=.flatpak-repo \
        .flatpak-builddir flatpak/de.swsnr.keepmeawake.yaml

install-flatpak: build-flatpak
    @# Install translated appstream metadata and desktop file
    install -Dm0644 build/de.swsnr.keepmeawake.metainfo.xml '/app/share/metainfo/{{APPID}}.metainfo.xml'
    install -Dm0644 build/de.swsnr.keepmeawake.desktop '/app/share/applications/{{APPID}}.desktop'
    @# Install configured entrypoint
    install -Dm0755 build/entrypoint.js '/app/bin/{{APPID}}'
    @# Install binary resource bundles
    install -Dm0644 -t '/app/share/{{APPID}}/resources' build/resources/*.gresource
    @# Copy compiled javascript files
    cp -rT build/js '/app/share/{{APPID}}/js'

format:
    npx prettier --write .
    blueprint-compiler format ui/**/*.blp --fix

lint-flatpak:
    flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest flatpak/de.swsnr.keepmeawake.yaml
    flatpak run --command=flatpak-builder-lint org.flatpak.Builder appstream de.swsnr.keepmeawake.metainfo.xml

lint-data:
    appstreamcli validate --strict --pedantic --explain de.swsnr.keepmeawake.metainfo.xml

lint: && lint-data lint-flatpak
    npx eslint .
    npx prettier --check .
    blueprint-compiler format ui/**/*.blp

test-all: && build lint
    npm ci

# Update NPM sources for flatpak.
flatpak-update-npm-sources:
    @# flatpak-node-generator says we should remove node_modules, see https://github.com/flatpak/flatpak-builder-tools/blob/master/node/README.md#usage-1
    rm -rf node_modules
    flatpak run --command=flatpak-node-generator org.flatpak.Builder \
        npm package-lock.json \
        -o flatpak/de.swsnr.keepmeawake.npm-sources.json
