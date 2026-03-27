# Quality Checks

Run the checks that match the area you touched.

## iOS

```bash
cd ios && just build
cd ios && just test-all
cd ios && just lint
cd ios && swiftformat .
```

## Android

```bash
cd android && ./gradlew test
cd android && ./gradlew lint
cd android && ./gradlew detekt
cd android && ./gradlew ktlintFormat
```

## Core

```bash
cd core && just test <CRATE>
cd core && cargo clippy -p <crate> -- -D warnings
cd core && just format
```

If you change shared models or bindings, also run the generation steps and validate both mobile apps.
