mod constants;
mod localization;
mod mocks;
mod models;
mod remote_mappers;
#[cfg(test)]
mod testkit;

use primitives::Platform;

use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

fn main() {
    let target = std::env::args().nth(1).expect("no platform specified");
    if target == "localize" {
        localization::generate(&std::env::args().skip(2).collect::<Vec<_>>()).unwrap();
        return;
    }
    let directory = std::env::args().nth(2).expect("no path specified");
    let platform = match target.as_str() {
        "ios" => Platform::IOS,
        "android" => Platform::Android,
        other => panic!("unsupported generator target: {other}"),
    };
    let models = models::generate(Path::new("."), platform, Path::new(&directory), &models::TypeMappings::from_yaml(remote_mappers::CONFIG));
    for (path, contents) in models.files {
        write_generated(&path, contents);
    }
    for path in models.stale {
        fs::remove_file(&path).unwrap_or_else(|error| panic!("failed to remove {}: {error}", path.display()));
    }
    generate_remote_mappers(platform, &directory);
    generate_constants(platform, &directory);
}

fn generate_constants(platform: Platform, platform_directory_path: &str) {
    let constants = constants::Constants::load(Path::new("."));
    let (contents, path) = match platform {
        Platform::IOS => (constants.swift(), format!("{platform_directory_path}/{}", constants::SWIFT_PATH)),
        Platform::Android => (constants.kotlin(), format!("{platform_directory_path}/{}", constants::KOTLIN_PATH)),
    };
    write_generated(Path::new(&path), contents);
}

fn generate_remote_mappers(platform: Platform, platform_directory_path: &str) {
    let generator = remote_mappers::Generator::load(Path::new("."));
    if generator.is_empty() {
        return;
    }
    write_generated(Path::new(remote_mappers::REMOTE_TYPES_PATH), generator.remote_types());

    let files = match platform {
        Platform::IOS => vec![
            (generator.swift(), format!("{platform_directory_path}/GemstonePrimitives/Sources/Generated/RemoteTypeMappers.swift")),
            (generator.swift_mocks(), format!("{platform_directory_path}/Primitives/TestKit/GeneratedMocks.swift")),
            (generator.swift_core_mocks(), format!("{platform_directory_path}/GemstonePrimitives/TestKit/GeneratedMocks.swift")),
        ],
        Platform::Android => vec![
            (generator.kotlin(), format!("{platform_directory_path}/../../gemwallet/android/ext/RemoteTypeMappers.kt")),
            (
                generator.kotlin_mocks(),
                format!("{platform_directory_path}/../../../../../testFixtures/kotlin/com/gemwallet/android/testkit/GeneratedMocks.kt"),
            ),
        ],
    };
    for (contents, path) in files {
        write_generated(Path::new(&path), contents);
    }
}

fn write_generated(path: &Path, contents: String) {
    let contents = if path.extension().is_some_and(|extension| extension == "rs") { format_rust(path, &contents) } else { contents };
    if fs::read_to_string(path).is_ok_and(|existing| existing == contents) {
        return;
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create generated directory");
    }
    fs::write(path, contents).expect("failed to write generated file");
}

fn format_rust(path: &Path, contents: &str) -> String {
    let child = Command::new("rustfmt").args(["--emit", "stdout"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("failed to run rustfmt");
    child.stdin.as_ref().expect("rustfmt stdin").write_all(contents.as_bytes()).expect("failed to write to rustfmt");
    let output = child.wait_with_output().expect("failed to wait for rustfmt");
    assert!(output.status.success(), "rustfmt failed on {}", path.display());
    String::from_utf8(output.stdout).expect("rustfmt output is not UTF-8")
}
