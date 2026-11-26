mod error;

use std::path::Path;

pub use error::ArgsError;

const HELP: [&str; 2] = ["--help", "-h"];

pub struct Args {
    program: String,
    arguments: Vec<String>,
    positionals: Vec<(String, String)>,
    options: Vec<(String, String)>,
    error: Option<String>,
}

impl Args {
    pub fn new(program: &str, arguments: impl IntoIterator<Item = String>) -> Self {
        Self {
            program: program.to_string(),
            arguments: arguments.into_iter().collect(),
            positionals: Vec::new(),
            options: Vec::new(),
            error: None,
        }
    }

    pub fn from_env() -> Self {
        let mut arguments = std::env::args();
        let program = arguments.next().unwrap_or_default();
        Self::new(Path::new(&program).file_name().and_then(|name| name.to_str()).unwrap_or_default(), arguments)
    }

    pub fn flag(&mut self, names: &[&str], help: &str) -> bool {
        self.options.push((label(names, None), help.to_string()));
        let count = self.arguments.len();
        self.arguments.retain(|argument| !names.contains(&argument.as_str()));
        self.arguments.len() < count
    }

    pub fn option(&mut self, names: &[&str], value: &str, help: &str) -> Option<String> {
        self.options.push((label(names, Some(value)), help.to_string()));
        self.take(names)
    }

    pub fn options(&mut self, names: &[&str], value: &str, help: &str) -> Vec<String> {
        self.options.push((label(names, Some(value)), help.to_string()));
        std::iter::from_fn(|| self.take(names)).collect()
    }

    pub fn required(&mut self, names: &[&str], value: &str, help: &str) -> String {
        self.option(names, value, help).unwrap_or_else(|| self.fail(format!("missing argument: {}", names.join(", "))))
    }

    pub fn positional(&mut self, name: &str, help: &str) -> String {
        self.positionals.push((format!("<{name}>"), help.to_string()));
        match self.arguments.iter().position(|argument| !argument.starts_with('-')) {
            Some(index) => self.arguments.remove(index),
            None => self.fail(format!("missing argument: <{name}>")),
        }
    }

    pub fn finish(mut self, about: &str) -> Result<(), ArgsError> {
        let help = self.flag(&HELP, "Print help");
        let usage = self.usage(about);
        match self.error.or_else(|| self.arguments.first().map(|argument| format!("unexpected argument: {argument}"))) {
            _ if help => Err(ArgsError::Help(usage)),
            Some(error) => Err(ArgsError::Invalid(format!("{error}\n\n{usage}"))),
            None => Ok(()),
        }
    }

    fn take(&mut self, names: &[&str]) -> Option<String> {
        let index = self.arguments.iter().position(|argument| names.contains(&argument.as_str()))?;
        let name = self.arguments.remove(index);
        match index < self.arguments.len() && !HELP.contains(&self.arguments[index].as_str()) {
            true => Some(self.arguments.remove(index)),
            false => Some(self.fail(format!("missing value for {name}"))),
        }
    }

    fn fail(&mut self, error: String) -> String {
        self.error.get_or_insert(error);
        String::new()
    }

    fn usage(&self, about: &str) -> String {
        let width = self.positionals.iter().chain(&self.options).map(|(label, _)| label.len()).max().unwrap_or_default();
        let section = |title: &str, lines: &[(String, String)]| match lines.is_empty() {
            true => String::new(),
            false => lines.iter().fold(format!("\n\n{title}:"), |section, (label, help)| format!("{section}\n  {label:<width$}  {help}").trim_end().to_string()),
        };
        let positionals: String = self.positionals.iter().map(|(label, _)| format!(" {label}")).collect();
        format!("{about}\n\nUsage: {} [OPTIONS]{positionals}{}{}", self.program, section("Arguments", &self.positionals), section("Options", &self.options))
    }
}

fn label(names: &[&str], value: Option<&str>) -> String {
    let short = names.iter().find(|name| !name.starts_with("--")).map_or("    ".to_string(), |name| format!("{name}, "));
    let long = names.iter().find(|name| name.starts_with("--")).copied().unwrap_or_default();
    match value {
        Some(value) => format!("{short}{long} <{value}>"),
        None => format!("{short}{long}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(arguments: &[&str]) -> Args {
        Args::new("tool", arguments.iter().map(|argument| argument.to_string()))
    }

    #[test]
    fn test_flag() {
        let mut args = args(&["--debug", "run"]);

        assert!(args.flag(&["--debug", "-d"], "Enable debug logging"));
        assert!(!args.flag(&["--quiet", "-q"], "Print less"));
        assert_eq!(args.positional("command", "Command to run"), "run");
        assert_eq!(args.finish("About"), Ok(()));
    }

    #[test]
    fn test_option() {
        let mut args = args(&["-o", "out", "--crate", "gemstone", "--language"]);

        assert_eq!(args.option(&["--out-dir", "-o"], "DIR", "Output"), Some("out".to_string()));
        assert_eq!(args.option(&["--crate"], "NAME", "Crate"), Some("gemstone".to_string()));
        assert_eq!(args.option(&["--source"], "PATH", "Source"), None);
        assert_eq!(args.option(&["--language"], "LANGUAGE", "Language"), Some(String::new()));
        assert!(args.finish("About").unwrap_err().to_string().starts_with("missing value for --language\n\nAbout"));
    }

    #[test]
    fn test_options() {
        let mut args = args(&["-H", "a:1", "ethereum", "--header", "b:2"]);

        assert_eq!(args.options(&["--header", "-H"], "NAME:VALUE", "Header"), vec!["a:1".to_string(), "b:2".to_string()]);
        assert_eq!(args.positional("chain", "Chain"), "ethereum");
        assert_eq!(args.finish("About"), Ok(()));
    }

    #[test]
    fn test_required() {
        let mut args = args(&[]);

        assert_eq!(args.required(&["--profile", "-p"], "PROFILE", "Profile"), "");
        assert!(args.finish("About").unwrap_err().to_string().starts_with("missing argument: --profile, -p\n\n"));
    }

    #[test]
    fn test_positional() {
        let mut args = args(&["ethereum"]);

        assert_eq!(args.positional("chain", "Chain"), "ethereum");
        assert_eq!(args.positional("url", "Node URL"), "");
        assert!(args.finish("About").unwrap_err().to_string().starts_with("missing argument: <url>\n\n"));
    }

    #[test]
    fn test_finish() {
        assert_eq!(args(&[]).finish("About"), Ok(()));
        assert!(args(&["--unknown"]).finish("About").unwrap_err().to_string().starts_with("unexpected argument: --unknown\n\n"));

        let mut args = args(&["--help"]);
        args.flag(&["--debug", "-d"], "Enable debug logging");
        args.option(&["--rpc-url"], "URL", "Node URL");
        args.required(&["--profile", "-p"], "PROFILE", "Checks to run");
        args.positional("chain", "Chain to check");

        assert_eq!(
            args.finish("Check a node"),
            Err(ArgsError::Help(
                "Check a node\n\nUsage: tool [OPTIONS] <chain>\n\nArguments:\n  <chain>                  Chain to check\n\nOptions:\n  -d, --debug              Enable debug logging\n      --rpc-url <URL>      Node URL\n  -p, --profile <PROFILE>  Checks to run\n  -h, --help               Print help"
                    .to_string()
            ))
        );
    }
}
