default:
    just --list

build:
    npx tsc --build

clean:
    rm -rf build

run: build
    gjs -m src/de.swsnr.keepmeawake.src.js

format:
    npx prettier --write .

lint:
    npx eslint .
    npx prettier --check .

test-all: build lint
