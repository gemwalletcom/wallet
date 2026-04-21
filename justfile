mod ios 'ios/justfile'
mod android 'android/justfile'

default:
    @just --list

setup-git:
    @echo "==> Setup git submodules"
    @git submodule sync --recursive
    @git submodule update --init --recursive
    @git config submodule.recurse true

build:
    @echo "==> Building iOS app"
    @just ios build
    @echo "==> Building Android app"
    @just android build

run-ios:
    @just ios run

run-android:
    @just android run

test:
    @echo "==> Test iOS app"
    @just ios test
    @echo "==> Test Android app"
    @just android test

test-integration:
    @echo "==> Test iOS app integration"
    @just ios test-integration
    @echo "==> Test Android app integration"
    @just android test-integration

generate: generate-models generate-stone

generate-models:
    @just ios generate-models
    @just android generate-models

generate-stone:
    @just ios generate-stone

localize:
    @just ios localize
    @just android localize

bump TARGET="patch":
    @bash ./scripts/bump.sh {{TARGET}}

core-upgrade:
    @git submodule update --recursive --remote
