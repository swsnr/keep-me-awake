APPID := "de.swsnr.keepmeawake.Devel"
VERSION := `git describe --always`

xgettext_opts := '--package-name=' + APPID + \
    ' --foreign-user --copyright-holder "Sebastian Wiesner <sebastian@swsnr.de>"' + \
    ' --sort-by-file --from-code=UTF-8 --add-comments'

default:
    just --list

# Extract messages from all source files.
pot:
    find src -name '*.rs' > po/POTFILES.rs
    find resources/ -name '*.blp' > po/POTFILES.blp
    xgettext {{xgettext_opts}} --language=Rust --keyword=dpgettext2:2c,3 --files-from=po/POTFILES.rs --output=po/de.swsnr.keepmeawake.rs.pot
    xgettext {{xgettext_opts}} --language=C --keyword=_ --keyword=C_:1c,2 --files-from=po/POTFILES.blp --output=po/de.swsnr.keepmeawake.blp.pot
    xgettext {{xgettext_opts}} --output=po/de.swsnr.keepmeawake.pot \
        po/de.swsnr.keepmeawake.rs.pot po/de.swsnr.keepmeawake.blp.pot \
        de.swsnr.keepmeawake.metainfo.xml de.swsnr.keepmeawake.desktop
    rm -f po/POTFILES* po/de.swsnr.keepmeawake.rs.pot po/de.swsnr.keepmeawake.blp.pot
    @# We strip the POT-Creation-Date from the resulting POT because xgettext bumps
    @# it everytime regardless if anything else changed, and this just generates
    @# needless diffs.
    sed -i /POT-Creation-Date/d po/de.swsnr.keepmeawake.pot

# Write APP ID to file for build
configure-app-id:
    @rm -f build/app-id
    @mkdir -p build
    @# Do not add a newline; that'd break the app ID in include_str!
    echo -n '{{APPID}}' > build/app-id

# Compile blueprint resources
compile-blueprint:
    @mkdir -p build/resources-src/ui build/resources
    blueprint-compiler batch-compile build/resources-src/ui resources/ui resources/ui/*.blp

# Compile translated desktop file.
compile-desktop-file:
    @mkdir -p build
    msgfmt --desktop --template de.swsnr.keepmeawake.desktop -d po \
        --output build/de.swsnr.keepmeawake.desktop
    @# Patch app ID
    sed -i '/{{APPID}}/! s/de\.swsnr\.keepmeawake/{{APPID}}/g' \
        build/de.swsnr.keepmeawake.desktop

# Compile translated metainfo file.
compile-metainfo:
    @mkdir -p build/resources-src
    msgfmt --xml --template de.swsnr.keepmeawake.metainfo.xml -d po \
        --output build/de.swsnr.keepmeawake.metainfo.xml
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

# Compile D-Bus service file.
compile-dbus-service:
    @mkdir -p build
    cp -t build dbus-1/de.swsnr.keepmeawake.service
    sed -i '/{{APPID}}/! s/de\.swsnr\.keepmeawake/{{APPID}}/g' \
        build/de.swsnr.keepmeawake.service

# Build the application.
compile: configure-app-id compile-resources compile-desktop-file compile-dbus-service

# Run in scope with correct app ID.
run *ARGS:
    systemd-run --user --scope --unit "app-{{APPID}}-dev" cargo run {{ARGS}}

# Clean build artifacts
clean:
    rm -rf build .flatpak-repo .flatpak-builddir

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

_install-po destprefix po_file:
    install -dm0755 '{{destprefix}}/share/locale/{{file_stem(po_file)}}/LC_MESSAGES'
    msgfmt -o '{{destprefix}}/share/locale/{{file_stem(po_file)}}/LC_MESSAGES/{{APPID}}.mo' '{{po_file}}'

# Install the app for flatpak packaging.
install-flatpak:
    @# Install all message catalogs (override VERSION to avoid redundant git describe calls)
    find po/ -name '*.po' -exec just VERSION= APPID='{{APPID}}' _install-po '/app' '{}' ';'
    @# Install translated appstream metadata and desktop file
    install -Dm0644 build/de.swsnr.keepmeawake.metainfo.xml '/app/share/metainfo/{{APPID}}.metainfo.xml'
    install -Dm0644 build/de.swsnr.keepmeawake.desktop '/app/share/applications/{{APPID}}.desktop'
    @# Install binary from cargo build --release
    install -Dm0755 target/release/keep-me-awake '/app/bin/{{APPID}}'
    @# Install configured files
    install -Dm0644 build/de.swsnr.keepmeawake.service '/app/share/dbus-1/services/{{APPID}}.service'
    @# Static files (icons, etc.)
    install -Dm0644 -t '/app/share/icons/hicolor/scalable/apps/' 'resources/icons/scalable/apps/{{APPID}}.svg'
    install -Dm0644 resources/icons/symbolic/apps/de.swsnr.keepmeawake-symbolic.svg \
        '/app/share/icons/hicolor/symbolic/apps/{{APPID}}-symbolic.svg'

# Lint blueprint UI definitions.
lint-blueprint:
    blueprint-compiler format resources/**/*.blp

# Lint flatpak manifest.
lint-flatpak:
    flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest flatpak/de.swsnr.keepmeawake.yaml
    flatpak run --command=flatpak-builder-lint org.flatpak.Builder appstream de.swsnr.keepmeawake.metainfo.xml

# Lint appstream data.
lint-data:
    appstreamcli validate --strict --pedantic --explain de.swsnr.keepmeawake.metainfo.xml

# Lint Rust code.
lint-rust:
    cargo +stable deny --all-features --locked check
    cargo +stable fmt -- --check
    cargo +stable clippy --all-targets

# Lint everything.
lint-all: lint-rust lint-blueprint lint-data lint-flatpak

# Vet the Rust supply chain.
vet *ARGS:
    cargo vet {{ARGS}}

# Test rust.
test-rust:
    cargo +stable build
    cargo +stable test

# Test everything.
test-all: (vet "--locked") lint-all compile test-rust

# Update Cargo sources for flatpak for `VERSION`.
flatpak-update-cargo-sources:
    flatpak run --command=flatpak-cargo-generator org.flatpak.Builder \
        <(git --no-pager show '{{VERSION}}:Cargo.lock') -o flatpak/de.swsnr.keepmeawake.cargo-sources.json

# Update the flatpak manifest for `VERSION`.
flatpak-update-manifest: flatpak-update-cargo-sources
    yq eval -i '.modules.[2].sources.[0].tag = "$TAG_NAME"' flatpak/de.swsnr.keepmeawake.yaml
    yq eval -i '.modules.[2].sources.[0].commit = "$TAG_COMMIT"' flatpak/de.swsnr.keepmeawake.yaml
    env TAG_NAME='{{VERSION}}' \
        TAG_COMMIT="$(git rev-parse '{{VERSION}}')" \
        yq eval -i '(.. | select(tag == "!!str")) |= envsubst' flatpak/de.swsnr.keepmeawake.yaml
    git add flatpak/de.swsnr.keepmeawake.yaml flatpak/de.swsnr.keepmeawake.cargo-sources.json
    @git commit -m 'Update flatpak manifest for {{VERSION}}'
    @echo "Run git push and trigger sync workflow at https://github.com/flathub/de.swsnr.keepmeawake/actions/workflows/sync.yaml"

# Print release notes
print-release-notes:
    @appstreamcli metainfo-to-news --format yaml de.swsnr.keepmeawake.metainfo.xml - | \
        yq eval-all '[.]' -oj | jq -r --arg tag "{{VERSION}}" \
        '.[] | select(.Version == ($tag | ltrimstr("v"))) | .Description | tostring'

_post-release:
    @echo "Create new release at https://codeberg.org/swsnr/keep-me-awake/tags"
    @echo "Run 'just print-release-notes' to get Markdown release notes for the release"
    @echo "Run 'just flatpak-update-manifest' to update the flatpak manifest."

release *ARGS: test-all && _post-release
    cargo release {{ARGS}}

# Assemble the README image from screenshots.
build-social-image:
    montage -geometry 602x602+19+19 \
        screenshots/inhibit-nothing.png screenshots/inhibit-suspend-and-idle.png \
        social-image.png
    oxipng social-image.png

# Run with default settings to make screenshots
run-for-screenshot:
    @# Run app with default settings: Force the in-memory gsettings backend to
    @# block access to standard Gtk settings, and tell Adwaita not to access
    @# portals to prevent it from getting dark mode and accent color from desktop
    @# settings.
    @#
    @# Effectively this makes our app run with default settings.
    env GSETTINGS_BACKEND=memory ADW_DISABLE_PORTAL=1 cargo run
