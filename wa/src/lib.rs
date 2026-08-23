#![cfg_attr(not(test), no_std)]

pub const WA_MAX_NAME_LEN: usize = 32;
pub const WA_MAX_HELP_LEN: usize = 256;
pub const WA_MAX_SUBCOMMANDS: usize = 16;

use heapless::String;
#[derive(Debug, Clone)]
pub struct WaArgument {
    pub name: String<WA_MAX_NAME_LEN>,
    pub help: String<WA_MAX_HELP_LEN>,
}
#[derive(Debug, Clone)]
pub struct WaCommand {
    pub name: String<WA_MAX_NAME_LEN>,
    pub help: String<WA_MAX_HELP_LEN>,
}
#[derive(Debug, Clone)]
pub struct Wa {
    pub name: String<WA_MAX_NAME_LEN>,
    pub help: String<WA_MAX_HELP_LEN>,
    pub subcommands: heapless::Vec<WaCommand, WA_MAX_SUBCOMMANDS>,
}

impl Wa {
    /// Creates a new `Wa` instance with the given name and help message.
    ///
    /// # Arguments
    ///
    /// * `name` - A `String` representing the name of the application. Must be a valid name according to the `is_valid_name` function.
    /// * `help` - A `String` representing the help message for the application. Must be a valid help message according to the `is_valid_help` function.
    ///
    /// # Returns
    ///
    /// A new instance of `Wa`.
    ///
    /// # Panics
    ///
    /// This function will panic if the provided name or help message is invalid.
    ///
    /// # Examples
    /// ```
    /// use wa::Wa;
    /// use wa::WA_MAX_NAME_LEN;
    /// use wa::WA_MAX_HELP_LEN;
    /// use heapless::String;
    /// use core::convert::TryFrom;
    /// use core::assert;
    /// use core::clone::Clone;
    /// let name = String::<WA_MAX_NAME_LEN>::try_from("myapp").expect("Invalid name");
    /// let help = String::<WA_MAX_HELP_LEN>::try_from("This is my application").expect("Invalid help");
    /// let wa = Wa::new(&name, &help);
    /// assert!(wa.name == name);
    /// assert!(wa.help == help);
    /// ```
    #[must_use]
    pub fn new(name: &String<WA_MAX_NAME_LEN>, help: &String<WA_MAX_HELP_LEN>) -> Self {
        Self {
            name: create_name(name.as_str()).expect("Invalid name"),
            help: create_help(help.as_str()).expect("Invalid help"),
            subcommands: heapless::Vec::new(),
        }
    }

    /// Adds a subcommand to the `Wa` instance.
    ///
    /// # Arguments
    ///
    /// * `command` - A `WaCommand` instance representing the subcommand to be added.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `Wa` instance, allowing for method chaining.
    ///
    /// # Panics
    ///
    /// This function will panic if the maximum number of subcommands (`WA_MAX_SUBCOMMANDS`) is exceeded.
    ///
    /// # Examples
    /// ```
    /// use wa::Wa;
    /// use wa::WaCommand;
    /// use wa::WA_MAX_NAME_LEN;
    /// use wa::WA_MAX_HELP_LEN;
    /// use heapless::String;
    /// use core::convert::TryFrom;
    /// use core::assert;
    /// let name = String::<WA_MAX_NAME_LEN>::try_from("myapp").expect("Invalid name");
    /// let help = String::<WA_MAX_HELP_LEN>::try_from("This is my application").expect("Invalid help");
    /// let mut wa = Wa::new(name, help);
    /// let subcommand_name = String::<WA_MAX_NAME_LEN>::try_from("sub").expect("Invalid subcommand name");
    /// let subcommand_help = String::<WA_MAX_HELP_LEN>::try_from("This is a subcommand").expect("Invalid subcommand help");
    /// let subcommand = WaCommand {
    ///     name: subcommand_name,
    ///     help: subcommand_help,
    /// };
    /// wa.subcommand(subcommand);
    /// assert!(wa.subcommands.len() == 1);
    /// ```
    pub fn subcommand(&mut self, command: WaCommand) -> &mut Self {
        self.subcommands
            .push(command)
            .expect("Failed to add subcommand");
        self
    }
}

/// Validates the name according to the specified rules.
///
/// # Arguments
/// * `name` - A string slice representing the name to be validated.
///
/// # Returns
/// * `true` if the name is valid, `false` otherwise.
#[must_use]
pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && name.len() <= WA_MAX_NAME_LEN
}

/// Creates a `String` from the provided name if it is valid.
///
/// # Arguments
/// * `name` - A string slice representing the name to be converted.
///
/// # Returns
/// * `Ok(String<WA_MAX_NAME_LEN>)` if the name is valid, `Err(())` otherwise.
fn create_name(name: &str) -> Result<String<WA_MAX_NAME_LEN>, ()> {
    if is_valid_name(name) {
        Ok(String::try_from(name).map_err(|_| ())?)
    } else {
        Err(())
    }
}

/// Validates the help message according to the specified rules.
///
/// # Arguments
/// * `help` - A string slice representing the help message to be validated.
///
/// # Returns
/// * `true` if the help message is valid, `false` otherwise.
#[must_use]
pub const fn is_valid_help(help: &str) -> bool {
    !help.is_empty() && help.len() <= WA_MAX_HELP_LEN
}

/// Creates a `String` from the provided help message if it is valid.
/// # Arguments
/// * `help` - A string slice representing the help message to be converted.
///
/// # Returns
/// * `Ok(String<WA_MAX_HELP_LEN>)` if the help message is valid, `Err(())` otherwise.
fn create_help(help: &str) -> Result<String<WA_MAX_HELP_LEN>, ()> {
    if is_valid_help(help) {
        Ok(String::try_from(help).map_err(|_| ())?)
    } else {
        Err(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wa() {
        let name = String::<WA_MAX_NAME_LEN>::try_from("myapp").unwrap();
        let help = String::<WA_MAX_HELP_LEN>::try_from("This is my application").unwrap();
        let wa = Wa::new(&name, &help);
        assert!(wa.name == name);
        assert!(wa.help == help);
    }

    #[test]
    fn test_is_valid_name() {
        assert!(is_valid_name("valid-name"));
        assert!(!is_valid_name("Invalid-Name"));
        assert!(!is_valid_name(""));
    }

    #[test]
    fn test_is_valid_help() {
        assert!(is_valid_help("This is a help message"));
        assert!(!is_valid_help(""));
    }

    #[test]
    fn test_create_name() {
        assert!(create_name("valid-name").is_ok());
        assert!(create_name("Invalid-Name").is_err());
        assert!(create_name("").is_err());
    }

    #[test]
    fn test_create_help() {
        assert!(create_help("This is a help message").is_ok());
        assert!(create_help("").is_err());
    }
}
