use core::fmt::{self, Write};

/// Maximum number of arguments in a [`Command`] definition.
pub const MAX_CONSOLE_ARGS: usize = 32;

/// Describes a command-line argument.
///
/// Arguments with a short or long name are options. Arguments without either
/// name are positional and are matched in definition order.
#[derive(Copy, Clone)]
pub struct Arg<'a> {
    id: &'a str,
    short: Option<char>,
    long: Option<&'a str>,
    help: &'a str,
    value_name: &'a str,
    takes_value: bool,
    required: bool,
}

impl<'a> Arg<'a> {
    /// Creates an argument with a stable identifier used to retrieve matches.
    ///
    /// Use [`long`](Self::long) or [`short`](Self::short) for an option.
    /// Without either, the argument is positional.
    #[must_use]
    pub const fn new(id: &'a str) -> Self {
        Self {
            id,
            short: None,
            long: None,
            help: "",
            value_name: "VALUE",
            takes_value: false,
            required: false,
        }
    }

    /// Sets the one-character short option name, such as `-v`.
    #[must_use]
    pub const fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    /// Sets the long option name, such as `--verbose`.
    #[must_use]
    pub const fn long(mut self, long: &'a str) -> Self {
        self.long = Some(long);
        self
    }

    /// Sets the help text displayed by [`Command::write_help`].
    #[must_use]
    pub const fn help(mut self, help: &'a str) -> Self {
        self.help = help;
        self
    }

    /// Declares that this option or positional argument takes a value.
    #[must_use]
    pub const fn takes_value(mut self, value_name: &'a str) -> Self {
        self.takes_value = true;
        self.value_name = value_name;
        self
    }

    /// Marks this argument as required.
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Command definition and argument parser, with no heap allocation.
pub struct Command<'a> {
    name: &'a str,
    about: &'a str,
    args: &'a [Arg<'a>],
    subcommands: &'a [Command<'a>],
}

impl<'a> Command<'a> {
    /// Creates a command definition.
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        Self {
            name,
            about: "",
            args: &[],
            subcommands: &[],
        }
    }

    /// Sets the short description shown in generated help.
    #[must_use]
    pub const fn about(mut self, about: &'a str) -> Self {
        self.about = about;
        self
    }

    /// Sets the command's argument definitions.
    #[must_use]
    pub const fn args(mut self, args: &'a [Arg<'a>]) -> Self {
        self.args = args;
        self
    }

    /// Sets the commands that may be selected after this command's options.
    #[must_use]
    pub const fn subcommands(mut self, subcommands: &'a [Command<'a>]) -> Self {
        self.subcommands = subcommands;
        self
    }

    /// Parses arguments, excluding the executable name.
    pub fn parse(&'a self, args: &'a [&'a str]) -> Result<Console<'a>, ParseError<'a>> {
        if self.args.len() > MAX_CONSOLE_ARGS {
            return Err(ParseError::TooManyDefinitions {
                maximum: MAX_CONSOLE_ARGS,
            });
        }
        validate_definition(self.args)?;
        for (index, command) in self.subcommands.iter().enumerate() {
            if command.name.is_empty()
                || self.subcommands[..index]
                    .iter()
                    .any(|previous| previous.name == command.name)
            {
                return Err(ParseError::InvalidDefinition(command.name));
            }
        }

        let mut console = Console {
            args,
            command: Some(self),
            present: [false; MAX_CONSOLE_ARGS],
            values: [None; MAX_CONSOLE_ARGS],
            subcommand: None,
        };
        let mut positional = 0;
        let mut index = 0;
        let mut positional_only = false;

        while index < args.len() {
            let token = args[index];
            if !positional_only && token == "--" {
                positional_only = true;
                index += 1;
                continue;
            }

            if !positional_only && token.starts_with("--") && token.len() > 2 {
                let (name, inline_value) = match token[2..].split_once('=') {
                    Some((name, value)) => (name, Some(value)),
                    None => (&token[2..], None),
                };
                let arg_index = self
                    .args
                    .iter()
                    .position(|arg| arg.long == Some(name))
                    .ok_or(ParseError::UnknownArgument(token))?;
                index += 1;
                consume_option(
                    &mut console,
                    arg_index,
                    self.args[arg_index].takes_value,
                    inline_value,
                    args,
                    &mut index,
                    token,
                )?;
                continue;
            }

            if !positional_only && token.starts_with('-') && token != "-" {
                let short_chars = token[1..].char_indices();
                let mut consumed_token = false;
                for (offset, short) in short_chars {
                    let arg_index = self
                        .args
                        .iter()
                        .position(|arg| arg.short == Some(short))
                        .ok_or(ParseError::UnknownArgument(token))?;
                    let arg = self.args[arg_index];
                    if arg.takes_value {
                        let value_start = 1 + offset + short.len_utf8();
                        let attached = (value_start < token.len()).then(|| &token[value_start..]);
                        index += 1;
                        consumed_token = true;
                        consume_option(
                            &mut console,
                            arg_index,
                            arg.takes_value,
                            attached,
                            args,
                            &mut index,
                            token,
                        )?;
                        break;
                    }
                    console.present[arg_index] = true;
                }
                if !consumed_token {
                    index += 1;
                }
                continue;
            }

            if let Some(subcommand) = self
                .subcommands
                .iter()
                .find(|command| command.name == token)
            {
                let subcommand_args = &args[index + 1..];
                subcommand.parse(subcommand_args)?;
                console.subcommand = Some((subcommand, subcommand_args));
                break;
            }

            let arg_index = next_positional(self.args, positional).ok_or({
                if self.subcommands.is_empty() {
                    ParseError::UnexpectedArgument(token)
                } else {
                    ParseError::UnknownSubcommand(token)
                }
            })?;
            positional = arg_index + 1;
            console.present[arg_index] = true;
            console.values[arg_index] = Some(token);
            index += 1;
        }

        for (arg_index, arg) in self.args.iter().enumerate() {
            if arg.required && !console.present[arg_index] {
                return Err(ParseError::MissingRequired(arg.id));
            }
        }
        Ok(console)
    }

    /// Writes a concise usage and argument list to any formatter.
    pub fn write_help(&self, output: &mut impl Write) -> fmt::Result {
        write!(output, "Usage: {}", self.name)?;
        for arg in self.args {
            output.write_char(' ')?;
            if arg.short.is_some() || arg.long.is_some() {
                if !arg.required {
                    output.write_char('[')?;
                }
                if let Some(short) = arg.short {
                    write!(output, "-{short}")?;
                    if arg.long.is_some() {
                        output.write_char('|')?;
                    }
                }
                if let Some(long) = arg.long {
                    write!(output, "--{long}")?;
                }
                if arg.takes_value {
                    write!(output, " <{}>", arg.value_name)?;
                }
                if !arg.required {
                    output.write_char(']')?;
                }
            } else if arg.required {
                write!(output, " <{}>", arg.id)?;
            } else {
                write!(output, " [{}]", arg.id)?;
            }
        }
        if !self.subcommands.is_empty() {
            output.write_str(" [COMMAND]")?;
        }
        output.write_char('\n')?;
        if !self.about.is_empty() {
            writeln!(output, "\n{}", self.about)?;
        }
        for arg in self.args {
            output.write_str("  ")?;
            if let Some(short) = arg.short {
                write!(output, "-{short}")?;
                if arg.long.is_some() {
                    output.write_str(", ")?;
                }
            }
            if let Some(long) = arg.long {
                write!(output, "--{long}")?;
            }
            if arg.takes_value {
                write!(output, " <{}>", arg.value_name)?;
            }
            if !arg.help.is_empty() {
                write!(output, "\t{}", arg.help)?;
            }
            output.write_char('\n')?;
        }
        if !self.subcommands.is_empty() {
            output.write_str("\nCommands:\n")?;
            for command in self.subcommands {
                write!(output, "  {}", command.name)?;
                if !command.about.is_empty() {
                    write!(output, "\t{}", command.about)?;
                }
                output.write_char('\n')?;
            }
        }
        Ok(())
    }
}

/// Error returned when a command definition or its arguments are invalid.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ParseError<'a> {
    /// The command definition exceeds [`MAX_CONSOLE_ARGS`].
    TooManyDefinitions { maximum: usize },
    /// An argument definition has invalid or conflicting names.
    InvalidDefinition(&'a str),
    /// The token does not match any declared option.
    UnknownArgument(&'a str),
    /// The token does not match any declared subcommand.
    UnknownSubcommand(&'a str),
    /// A positional token has no matching positional definition.
    UnexpectedArgument(&'a str),
    /// An option that takes a value was not given one.
    MissingValue(&'a str),
    /// A flag was given a value.
    UnexpectedValue(&'a str),
    /// A required argument was not supplied.
    MissingRequired(&'a str),
}

/// Parsed command-line arguments.
///
/// `Console::new` remains available for simple raw argument inspection.
pub struct Console<'a> {
    args: &'a [&'a str],
    command: Option<&'a Command<'a>>,
    present: [bool; MAX_CONSOLE_ARGS],
    values: [Option<&'a str>; MAX_CONSOLE_ARGS],
    subcommand: Option<(&'a Command<'a>, &'a [&'a str])>,
}

impl<'a> Console<'a> {
    /// Creates a raw argument view, excluding the executable name.
    #[must_use]
    pub const fn new(args: &'a [&'a str]) -> Self {
        Self {
            args,
            command: None,
            present: [false; MAX_CONSOLE_ARGS],
            values: [None; MAX_CONSOLE_ARGS],
            subcommand: None,
        }
    }

    /// Returns all input arguments in their original order.
    #[must_use]
    pub const fn args(&self) -> &'a [&'a str] {
        self.args
    }

    /// Returns the raw argument at `index`, or `None` if it is out of bounds.
    #[must_use]
    pub fn argument(&self, index: usize) -> Option<&'a str> {
        self.args.get(index).copied()
    }

    /// Returns whether the exact raw flag is present.
    #[must_use]
    pub fn has_flag(&self, flag: &str) -> bool {
        self.args.contains(&flag)
    }

    /// Returns a raw option value from either `--name value` or `--name=value`.
    #[must_use]
    pub fn option(&self, name: &str) -> Option<&'a str> {
        for (index, arg) in self.args.iter().enumerate() {
            if let Some(value) = arg
                .strip_prefix(name)
                .and_then(|suffix| suffix.strip_prefix('='))
            {
                return Some(value);
            }
            if *arg == name {
                return self.args.get(index + 1).copied();
            }
        }
        None
    }

    /// Returns whether a declared argument was supplied.
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.argument_index(id)
            .is_some_and(|index| self.present[index])
    }

    /// Returns the value of a declared value-taking argument.
    #[must_use]
    pub fn value(&self, id: &str) -> Option<&'a str> {
        self.argument_index(id).and_then(|index| self.values[index])
    }

    /// Returns whether a declared flag was supplied.
    #[must_use]
    pub fn flag(&self, id: &str) -> bool {
        self.contains(id)
    }

    /// Returns the selected subcommand name, if any.
    #[must_use]
    pub fn subcommand_name(&self) -> Option<&'a str> {
        self.subcommand.map(|(command, _)| command.name)
    }

    /// Parses and returns matches for the selected subcommand.
    ///
    /// The subcommand was validated when this `Console` was created.
    pub fn subcommand_matches(&self) -> Result<Option<Console<'a>>, ParseError<'a>> {
        let Some((command, args)) = self.subcommand else {
            return Ok(None);
        };
        command.parse(args).map(Some)
    }

    fn argument_index(&self, id: &str) -> Option<usize> {
        self.command?.args.iter().position(|arg| arg.id == id)
    }
}

fn validate_definition<'a>(args: &[Arg<'a>]) -> Result<(), ParseError<'a>> {
    for (index, arg) in args.iter().enumerate() {
        let is_positional = arg.short.is_none() && arg.long.is_none();
        if arg.id.is_empty()
            || (is_positional && !arg.takes_value)
            || (!is_positional && arg.takes_value && arg.value_name.is_empty())
            || arg.short.is_some_and(|short| short == '-')
            || arg
                .long
                .is_some_and(|long| long.is_empty() || long.starts_with('-'))
        {
            return Err(ParseError::InvalidDefinition(arg.id));
        }
        for other in &args[..index] {
            if arg.id == other.id
                || (arg.short.is_some() && arg.short == other.short)
                || (arg.long.is_some() && arg.long == other.long)
            {
                return Err(ParseError::InvalidDefinition(arg.id));
            }
        }
    }
    Ok(())
}

fn next_positional(args: &[Arg<'_>], start: usize) -> Option<usize> {
    args.iter()
        .enumerate()
        .skip(start)
        .find(|(_, arg)| arg.short.is_none() && arg.long.is_none())
        .map(|(index, _)| index)
}

fn consume_option<'a>(
    console: &mut Console<'a>,
    arg_index: usize,
    takes_value: bool,
    inline_value: Option<&'a str>,
    args: &'a [&'a str],
    index: &mut usize,
    token: &'a str,
) -> Result<(), ParseError<'a>> {
    if takes_value {
        let value = match inline_value {
            Some(value) => value,
            None => {
                let value = args
                    .get(*index)
                    .copied()
                    .ok_or(ParseError::MissingValue(token))?;
                if value == "--" {
                    return Err(ParseError::MissingValue(token));
                }
                *index += 1;
                value
            }
        };
        console.values[arg_index] = Some(value);
    } else if inline_value.is_some() {
        return Err(ParseError::UnexpectedValue(token));
    }
    console.present[arg_index] = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Arg, Command, Console, ParseError};

    #[test]
    fn reads_raw_arguments_and_options() {
        let args = ["build", "--release", "src/main.rs", "--target=x86_64"];
        let console = Console::new(&args);

        assert_eq!(console.args(), &args);
        assert_eq!(console.argument(0), Some("build"));
        assert_eq!(console.argument(4), None);
        assert!(console.has_flag("--release"));
        assert_eq!(console.option("--target"), Some("x86_64"));
    }

    #[test]
    fn parses_options_flags_and_positionals() {
        let definitions = [
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Show details"),
            Arg::new("config")
                .short('c')
                .long("config")
                .takes_value("FILE")
                .required(),
            Arg::new("input").takes_value("INPUT"),
        ];
        let command = Command::new("demo").args(&definitions);
        let args = ["-v", "--config=app.toml", "input.txt"];
        let console = command.parse(&args).unwrap();

        assert!(console.flag("verbose"));
        assert_eq!(console.value("config"), Some("app.toml"));
        assert_eq!(console.value("input"), Some("input.txt"));
        assert!(!console.contains("input-missing"));
    }

    #[test]
    fn parses_combined_short_flags_and_attached_values() {
        let definitions = [
            Arg::new("all").short('a'),
            Arg::new("verbose").short('v'),
            Arg::new("output").short('o').takes_value("FILE"),
        ];
        let command = Command::new("demo").args(&definitions);
        let args = ["-avoresult.txt"];
        let console = command.parse(&args).unwrap();

        assert!(console.flag("all"));
        assert!(console.flag("verbose"));
        assert_eq!(console.value("output"), Some("result.txt"));
    }

    #[test]
    fn stops_option_parsing_after_double_dash() {
        let definitions = [
            Arg::new("verbose").short('v'),
            Arg::new("input").takes_value("INPUT"),
        ];
        let command = Command::new("demo").args(&definitions);
        let args = ["--", "-v"];
        let console = command.parse(&args).unwrap();

        assert!(!console.flag("verbose"));
        assert_eq!(console.value("input"), Some("-v"));
    }

    #[test]
    fn reports_invalid_and_missing_arguments() {
        let definitions = [Arg::new("config")
            .long("config")
            .takes_value("FILE")
            .required()];
        let command = Command::new("demo").args(&definitions);

        assert!(matches!(
            command.parse(&["--unknown"]),
            Err(ParseError::UnknownArgument("--unknown"))
        ));
        assert!(matches!(
            command.parse(&[]),
            Err(ParseError::MissingRequired("config"))
        ));
        assert!(matches!(
            command.parse(&["--config"]),
            Err(ParseError::MissingValue("--config"))
        ));
        assert!(matches!(
            command.parse(&["--config", "x", "extra"]),
            Err(ParseError::UnexpectedArgument("extra"))
        ));
    }

    #[test]
    fn writes_command_help_without_allocation() {
        let definitions = [
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Show details"),
            Arg::new("config")
                .long("config")
                .takes_value("FILE")
                .required()
                .help("Configuration path"),
        ];
        let command = Command::new("demo")
            .about("Demo application")
            .args(&definitions);
        let mut help = String::new();
        command.write_help(&mut help).unwrap();

        assert!(help.contains("Usage: demo [-v|--verbose] --config <FILE>"));
        assert!(help.contains("Demo application"));
        assert!(help.contains("Configuration path"));
    }

    #[test]
    fn parses_subcommand_and_exposes_child_matches() {
        let clone_args = [
            Arg::new("repository").takes_value("URL").required(),
            Arg::new("mirror").long("mirror"),
        ];
        let status_args = [Arg::new("short").short('s').long("short")];
        let commands = [
            Command::new("clone")
                .about("Clone a repository")
                .args(&clone_args),
            Command::new("status")
                .about("Show repository status")
                .args(&status_args),
        ];
        let root = Command::new("wuu")
            .about("Version control tool")
            .subcommands(&commands);
        let argv = ["clone", "https://example.test/repo", "--mirror"];

        let matches = root.parse(&argv).unwrap();
        assert_eq!(matches.subcommand_name(), Some("clone"));
        let child = matches.subcommand_matches().unwrap().unwrap();
        assert_eq!(child.value("repository"), Some("https://example.test/repo"));
        assert!(child.has_flag("--mirror"));
        assert_eq!(child.argument(1), Some("--mirror"));
    }

    #[test]
    fn reports_unknown_subcommands_and_child_argument_errors() {
        let commands = [Command::new("status")];
        let root = Command::new("wuu").subcommands(&commands);

        assert!(matches!(
            root.parse(&["unknown"]),
            Err(ParseError::UnknownSubcommand("unknown"))
        ));
        assert!(matches!(
            root.parse(&["status", "--bad"]),
            Err(ParseError::UnknownArgument("--bad"))
        ));
    }

    #[test]
    fn includes_subcommands_in_help() {
        let commands = [
            Command::new("status").about("Show status"),
            Command::new("commit").about("Create a commit"),
        ];
        let root = Command::new("wuu").subcommands(&commands);
        let mut help = String::new();
        root.write_help(&mut help).unwrap();

        assert!(help.contains("Usage: wuu [COMMAND]"));
        assert!(help.contains("Commands:\n  status\tShow status\n  commit\tCreate a commit"));
    }

    #[test]
    fn raw_option_preserves_empty_inline_values() {
        let args = ["--name="];
        let console = Console::new(&args);
        assert_eq!(console.option("--name"), Some(""));
    }
}
