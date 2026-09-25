mod constants;
mod localization;
mod remote_mappers;
#[cfg(test)]
mod testkit;

use primitives::Platform;

use std::{
    fs::{self, DirEntry},
    io::Write,
    path::Path,
    process::{Command, Stdio},
    vec,
};

const ANDROID_PACKAGE_PREFIX: &str = "com.wallet.core";
const IOS_GENERATED_DIR: &str = "Generated";
const LANGUAGE_SWIFT: &str = "swift";
const LANGUAGE_KOTLIN: &str = "kotlin";
const LANG_KOTLIN_ETX: &str = "kt";
const LANGUAGE_TYPESCRIPT: &str = "typescript";
const LANG_TYPESCRIPT_EXT: &str = "ts";

#[derive(Clone, Copy)]
enum GeneratorType {
    Swift,
    Kotlin,
    TypeScript,
}

impl GeneratorType {
    fn ignored_files(&self) -> Vec<&'static str> {
        match self {
            Self::Swift => vec!["quote_asset.rs"],
            Self::Kotlin => vec!["asset_data.rs", "quote_asset.rs"],
            Self::TypeScript => vec!["transaction_input_type.rs"],
        }
    }
}

fn main() {
    let folders = vec!["crates/primitives"];

    let platform_str = std::env::args().nth(1).expect("no platform specified");
    if platform_str == "localize" {
        let args = std::env::args().skip(2).collect::<Vec<_>>();
        localization::generate(&args).unwrap();
        return;
    }

    let platform_directory_path = std::env::args().nth(2).expect("no path specified");

    let generator_type = match platform_str.as_str() {
        "web" => GeneratorType::TypeScript,
        "ios" => GeneratorType::Swift,
        "android" => GeneratorType::Kotlin,
        other => panic!("unsupported generator target: {other}"),
    };

    let mut ignored_files: Vec<&'static str> = [
        "lib.rs",
        "mod.rs",
        "client.rs",
        "model.rs",
        "address.rs",
        "address_formatter.rs",
        "big_int_hex.rs",
        "hash.rs",
        "pubkey.rs",
        "ethereum_address.rs",
        "keccak.rs",
        "number_formatter.rs",
        "mode.rs",
        "quote.rs",
        "slippage.rs",
    ]
    .to_vec();
    let mut platform_ignored = generator_type.ignored_files();
    ignored_files.append(&mut platform_ignored);

    for folder in folders {
        let src_path = format!("{folder}/src");
        let paths = get_paths(folder, src_path);
        process_paths(paths, folder, &generator_type, &platform_directory_path, &ignored_files);
    }

    generate_remote_mappers(&generator_type, &platform_directory_path);
    generate_constants(&generator_type, &platform_directory_path);
}

fn generate_constants(generator_type: &GeneratorType, platform_directory_path: &str) {
    let constants = constants::Constants::load(Path::new("."));
    let (contents, path) = match generator_type {
        GeneratorType::Swift => (constants.swift(), format!("{platform_directory_path}/{}", constants::SWIFT_PATH)),
        GeneratorType::Kotlin => (constants.kotlin(), format!("{platform_directory_path}/{}", constants::KOTLIN_PATH)),
        GeneratorType::TypeScript => return,
    };
    write_generated(&path, contents);
}

fn generate_remote_mappers(generator_type: &GeneratorType, platform_directory_path: &str) {
    let generator = remote_mappers::Generator::load(Path::new("."));
    if generator.is_empty() {
        return;
    }
    write_generated(remote_mappers::REMOTE_TYPES_PATH, generator.remote_types());

    let files = match generator_type {
        GeneratorType::Swift => [
            (generator.swift(), format!("{platform_directory_path}/GemstonePrimitives/Sources/Generated/RemoteTypeMappers.swift")),
            (generator.swift_mocks(), format!("{platform_directory_path}/Primitives/TestKit/GeneratedMocks.swift")),
        ],
        GeneratorType::Kotlin => [
            (generator.kotlin(), format!("{platform_directory_path}/../../gemwallet/android/ext/RemoteTypeMappers.kt")),
            (
                generator.kotlin_mocks(),
                format!("{platform_directory_path}/../../../../../testFixtures/kotlin/com/gemwallet/android/testkit/GeneratedMocks.kt"),
            ),
        ],
        GeneratorType::TypeScript => return,
    };
    for (contents, path) in files {
        write_generated(&path, contents);
    }
}

fn write_generated(path: &str, contents: String) {
    let path = Path::new(path);
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
    let mut child = Command::new("rustfmt").args(["--emit", "stdout"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("failed to run rustfmt");
    child.stdin.take().expect("rustfmt stdin").write_all(contents.as_bytes()).expect("failed to write to rustfmt");
    let output = child.wait_with_output().expect("failed to wait for rustfmt");
    assert!(output.status.success(), "rustfmt failed on {}", path.display());
    String::from_utf8(output.stdout).expect("rustfmt output is not UTF-8")
}

fn process_paths(paths: Vec<String>, _folder: &str, generator_type: &GeneratorType, platform_directory_path: &str, ignored_files: &[&str]) {
    let mut commands = vec![];
    for path in paths {
        // Example path:
        // ./crates/primitives/src/utxo.rs
        let vec: Vec<&str> = path.split("/src/").collect();
        if vec.len() < 2 {
            continue;
        }

        let first_parts: Vec<&str> = vec[0].split('/').collect();
        if first_parts.len() < 2 {
            continue;
        }

        let module_name = first_parts[1];

        let directory_paths: Vec<&str> = vec[1].split('/').collect();
        let mut directory_paths_capitalized = directory_paths.iter().filter(|x| !x.starts_with('.')).map(|&x| str_capitlize(x)).collect::<Vec<_>>();

        if directory_paths_capitalized.is_empty() {
            continue;
        }

        let file_path = directory_paths_capitalized.pop().unwrap();

        let file_name_original = directory_paths.last().unwrap_or(&"");

        if ignored_files.contains(file_name_original) {
            continue;
        }
        let input_path = format!("./{}/src/{}", vec[0], directory_paths.join("/"));
        if !fs::read_to_string(&input_path).expect("failed to read source file").contains("#[typeshare") {
            continue;
        }

        match generator_type {
            GeneratorType::Swift => {
                let ios_new_file_name = file_name(&file_path, LANGUAGE_SWIFT);
                let ios_new_path = format!("{}/{}", directory_paths_capitalized.join("/"), ios_new_file_name);
                let ios_output_path = output_path(Platform::IOS, platform_directory_path, str_capitlize(module_name).as_str(), ios_new_path);
                commands.push(typeshare_command(LANGUAGE_SWIFT, input_path.as_str(), ios_output_path.as_str(), None));
            }
            GeneratorType::Kotlin => {
                let kt_new_file_name = file_name(&file_path, LANG_KOTLIN_ETX);
                let directory_paths_lowercased: Vec<String> = directory_paths_capitalized.iter().map(|x| x.to_lowercase()).collect();
                let kt_new_path = format!("{}/{}", directory_paths_lowercased.join("/"), kt_new_file_name);
                let android_output_path = output_path(Platform::Android, platform_directory_path, module_name, kt_new_path.clone());
                let directory_package = directory_paths_lowercased.join(".");
                let android_package_name = format!("{}.{}{}", ANDROID_PACKAGE_PREFIX, module_name, if directory_package.is_empty() { String::new() } else { format!(".{directory_package}") });
                commands.push(typeshare_command(LANGUAGE_KOTLIN, input_path.as_str(), android_output_path.as_str(), Some(android_package_name.as_str())));
            }
            GeneratorType::TypeScript => {
                let ts_new_file_name = file_name(&file_path, LANG_TYPESCRIPT_EXT);
                let directory_paths_lowercased: Vec<String> = directory_paths_capitalized.iter().map(|x| x.to_lowercase()).collect();
                let ts_new_path = format!("{}/{}", directory_paths_lowercased.join("/"), ts_new_file_name);
                let web_output_path = output_path_web(platform_directory_path, module_name, ts_new_path);
                commands.push(typeshare_command(LANGUAGE_TYPESCRIPT, input_path.as_str(), web_output_path.as_str(), None));
            }
        }
    }
    run_typeshare(commands);
}

fn output_path(platform: Platform, directory: &str, module_name: &str, path: String) -> String {
    match platform {
        Platform::IOS => format!("{directory}/{module_name}/Sources/{IOS_GENERATED_DIR}/{path}"),
        Platform::Android => format!("{directory}/{module_name}/generated/{path}"),
    }
}

fn output_path_web(directory: &str, module_name: &str, path: String) -> String {
    format!("{directory}/{module_name}/{path}")
}

fn file_name(name: &str, file_extension: &str) -> String {
    let split: Vec<&str> = name.split('.').collect();
    let new_split: Vec<&str> = split[0].split('_').collect();
    let new_name = new_split.iter().map(|&x| str_capitlize(x)).collect::<Vec<_>>().join("");
    format!("{new_name}.{file_extension}")
}

fn typeshare_command(language: &str, input_path: &str, output_path: &str, package_name: Option<&str>) -> Command {
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent).expect("failed to create generated directory");
    }

    let mut command = Command::new("typeshare");
    command.arg(input_path).arg(format!("--lang={language}")).arg(format!("--output-file={output_path}"));

    if let Some(package_name) = package_name {
        command.arg(format!("--java-package={package_name}"));
    }
    command.stdout(Stdio::null()).stderr(Stdio::piped());
    command
}

fn run_typeshare(mut commands: Vec<Command>) {
    let parallelism = std::thread::available_parallelism().map_or(1, usize::from);
    for batch in commands.chunks_mut(parallelism) {
        let children: Vec<_> = batch.iter_mut().map(|command| (format!("{command:?}"), command.spawn().expect("failed to run typeshare"))).collect();
        for (command, child) in children {
            let output = child.wait_with_output().expect("failed to wait for typeshare");
            assert!(output.status.success(), "{command} failed: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
}

fn get_paths(_folder: &str, path: String) -> Vec<String> {
    let paths = match fs::read_dir(&path) {
        Ok(paths) => paths,
        Err(_) => {
            eprintln!("Warning: Could not read directory: {path}");
            return vec![];
        }
    };
    let mut result: Vec<String> = vec![];

    for path in paths {
        let dir_entry = path.unwrap();
        if dir_entry.path().is_dir() {
            let path_recursive = get_paths(_folder, clear_path(dir_entry));
            result.extend(path_recursive)
        } else {
            result.push(clear_path(dir_entry));
        }
    }

    result
}

fn clear_path(path: DirEntry) -> String {
    format!("{}", path.path().display())
}

fn str_capitlize(s: &str) -> String {
    format!("{}{}", s[..1].to_string().to_uppercase(), &s[1..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_name() {
        assert_eq!(file_name("token.rs", LANGUAGE_SWIFT), "Token.swift");
        assert_eq!(file_name("token_type.rs", LANGUAGE_SWIFT), "TokenType.swift");
    }

    #[test]
    fn test_str_capitlize() {
        assert_eq!(str_capitlize("balance"), "Balance");
        assert_eq!(str_capitlize("Balance"), "Balance");
    }
}
