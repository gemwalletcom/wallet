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
    @echo "==> Building iOS"
    @cd ios && just build-for-testing
    @echo "==> Building Android"
    @cd android && just build-test

generate: generate-models generate-stone

generate-models:
    @cd ios && just generate-model
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
