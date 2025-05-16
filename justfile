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
    mkdir -p build/ui build/resources
    blueprint-compiler batch-compile build/ui ui ui/*.blp
    glib-compile-resources --sourcedir=build/ui \
        --target build/resources/resources.ui.gresource \
        ui/resources.ui.gresource.xml

# Build the application.
build: compile-tsc compile-blueprint

# Clean build artifacts
clean:
    rm -rf build

# Run the application
run: build
    gjs -m build/js/run.js

format:
    npx prettier --write .
    blueprint-compiler format ui/**/*.blp --fix

lint:
    npx eslint .
    npx prettier --check .
    blueprint-compiler format ui/**/*.blp

test-all: build lint
