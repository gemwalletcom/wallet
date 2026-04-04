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
    @cd ios && just build
    @echo "==> Building Android app"
    @cd android && just build

test:
    @echo "==> Test iOS app"
    @cd ios && just test
    @echo "==> Test Android app"
    @cd android && just test

test-integration:
    @echo "==> Test iOS app integration"
    @cd ios && just test-integration
    @echo "==> Test Android app integration"
    @cd android && just test-integration

generate: generate-models generate-stone

generate-models:
    @cd ios && just generate-models
    @cd android && just generate-models

generate-stone:
    @cd ios && just generate-stone

localize:
    @cd ios && just localize-all
    @cd android && just localize

bump TARGET="patch":
    @bash ./scripts/bump.sh {{TARGET}}

core-upgrade:
    @git submodule update --recursive --remote
