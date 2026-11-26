mod error;

pub use error::ArgsError;

pub struct Args(Vec<String>);

impl Args {
    pub fn new(args: impl IntoIterator<Item = String>) -> Self {
        Self(args.into_iter().collect())
    }

    pub fn from_env() -> Self {
        Self::new(std::env::args().skip(1))
    }

    pub fn flag(&mut self, names: &[&str]) -> bool {
        let count = self.0.len();
        self.0.retain(|argument| !names.contains(&argument.as_str()));
        self.0.len() < count
    }

    pub fn option(&mut self, names: &[&str]) -> Result<Option<String>, ArgsError> {
        let Some(index) = self.0.iter().position(|argument| names.contains(&argument.as_str())) else {
            return Ok(None);
        };
        let name = self.0.remove(index);
        match index < self.0.len() {
            true => Ok(Some(self.0.remove(index))),
            false => Err(ArgsError::MissingValue(name)),
        }
    }

    pub fn options(&mut self, names: &[&str]) -> Result<Vec<String>, ArgsError> {
        let mut values = Vec::new();
        while let Some(value) = self.option(names)? {
            values.push(value);
        }
        Ok(values)
    }

    pub fn required(&mut self, names: &[&str]) -> Result<String, ArgsError> {
        self.option(names)?.ok_or_else(|| ArgsError::Missing(names.join(", ")))
    }

    pub fn positional(&mut self, name: &str) -> Result<String, ArgsError> {
        match self.0.iter().position(|argument| !argument.starts_with('-')) {
            Some(index) => Ok(self.0.remove(index)),
            None => Err(ArgsError::Missing(name.to_string())),
        }
    }

    pub fn finish(self) -> Result<(), ArgsError> {
        match self.0.into_iter().next() {
            Some(argument) => Err(ArgsError::Unexpected(argument)),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Args {
        Args::new(values.iter().map(|value| value.to_string()))
    }

    #[test]
    fn test_flag() {
        let mut args = args(&["--debug", "run"]);

        assert!(args.flag(&["--debug", "-d"]));
        assert!(!args.flag(&["--debug", "-d"]));
        assert_eq!(args.positional("command"), Ok("run".to_string()));
    }

    #[test]
    fn test_option() {
        let mut args = args(&["generate", "-o", "out", "--crate", "gemstone", "--language"]);

        assert_eq!(args.option(&["--out-dir", "-o"]), Ok(Some("out".to_string())));
        assert_eq!(args.option(&["--out-dir", "-o"]), Ok(None));
        assert_eq!(args.option(&["--crate"]), Ok(Some("gemstone".to_string())));
        assert_eq!(args.option(&["--language"]), Err(ArgsError::MissingValue("--language".to_string())));
    }

    #[test]
    fn test_options() {
        let mut args = args(&["-H", "a:1", "ethereum", "--header", "b:2"]);

        assert_eq!(args.options(&["--header", "-H"]), Ok(vec!["a:1".to_string(), "b:2".to_string()]));
        assert_eq!(args.options(&["--header", "-H"]), Ok(vec![]));
        assert_eq!(args.positional("chain"), Ok("ethereum".to_string()));
    }

    #[test]
    fn test_required() {
        let mut args = args(&["--profile", "full"]);

        assert_eq!(args.required(&["--profile", "-p"]), Ok("full".to_string()));
        assert_eq!(args.required(&["--profile", "-p"]), Err(ArgsError::Missing("--profile, -p".to_string())));
    }

    #[test]
    fn test_positional() {
        let mut args = args(&["ethereum", "--unknown", "https://example.com"]);

        assert_eq!(args.positional("chain"), Ok("ethereum".to_string()));
        assert_eq!(args.positional("url"), Ok("https://example.com".to_string()));
        assert_eq!(args.positional("extra"), Err(ArgsError::Missing("extra".to_string())));
    }

    #[test]
    fn test_finish() {
        assert_eq!(args(&[]).finish(), Ok(()));
        assert_eq!(args(&["--unknown"]).finish(), Err(ArgsError::Unexpected("--unknown".to_string())));
    }
}
