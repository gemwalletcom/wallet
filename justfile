default:
    @just --list

setup-git:
    @echo "==> Setup git submodules"
    @git submodule sync --recursive
    @git submodule update --init --recursive
    @git config submodule.recurse true

ios-bootstrap:
    @cd ios && just bootstrap

ios-build:
    @cd ios && just build-for-testing

ios-test:
    @cd ios && just test-without-building

android-bootstrap:
    @cd android && just bootstrap

android-build-test:
    @cd android && just build-test

android-test:
    @cd android && just test

generate: generate-model generate-stone

generate-model: generate-models

generate-models:
    @cd ios && just generate-model
    @cd android && just generate-models

generate-stone:
    @cd ios && just generate-stone

bump TARGET="patch":
    @bash ./scripts/bump.sh {{TARGET}}

core-upgrade:
    @git submodule update --recursive --remote
