use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::fs::File;
use std::io::{Read, BufReader, BufRead};

/// Generates and prints a manpage for the given binary to the screen.
///
/// # Arguments
/// * `binary_path` - Path to the binary for which the manpage is being generated.
/// * `name` - Name of the binary (used in the manpage header).
/// * `section` - Section number of the manpage (e.g., 1 for general commands, 2 for system calls).
/// * `title` - Title of the manpage (e.g., "General commands").
///
/// # Returns
/// * `Ok(())` if the manpage is successfully generated.
/// * `Err(String)` containing an error message if the generation fails.
pub fn generate_manpage(binary_path: &PathBuf, name: &str, section: u8, title: &str) -> Result<(), String> {

    // Generate manpage content
    let manpage_content = generate_manpage_content(binary_path, name, section, title)?;

    // Print manpage content to the screen
    println!("{}", manpage_content);
    Ok(())
}

/// Generates the content of the manpage based on the binary's help and version output.
///
/// # Arguments
/// * `binary_path` - Path to the binary.
/// * `name` - Name of the binary.
/// * `section` - Section number of the manpage.
/// * `title` - Title of the manpage.
///
/// # Returns
/// * `Ok(String)` containing the manpage content.
/// * `Err(String)` containing an error message if content generation fails.
fn generate_manpage_content(binary_path: &PathBuf, name: &str, section: u8, title: &str) -> Result<String, String> {
    let main_help = get_command_output(binary_path, &["--help"])?;
    let version = get_command_output(binary_path, &["--version"]).unwrap_or_else(|_| "1.0.0".to_string());
    let subcommands = get_subcommands(binary_path)?;

    let mut manpage = String::new();

    // Header
    manpage.push_str(&format!(
        ".TH \"{0}\" \"{1}\" \"\" \"{0} {2}\" \"{3}\"
",
        name.to_uppercase(),
        section,
        version.trim(),
        title,
    ));

    // Name section
    manpage.push_str(".SH NAME\n");
    manpage.push_str(&format!("{} \\- {}\n", name, get_short_description(&main_help)));

    // Synopsis section
    manpage.push_str(".SH SYNOPSIS\n");
    manpage.push_str(".B ");
    manpage.push_str(name);
    manpage.push_str("\n");

    // Extract the usage and ensure no duplication of the binary name
    if let Some(usage) = get_usage(&main_help) {
        let cleaned_usage = usage
            .trim()
            .replace(&format!("{} ", name), ""); // Remove duplicate binary name
        manpage.push_str(&cleaned_usage);
    } else {
        manpage.push_str("[OPTIONS] <BINARY_PATH>");
    }
    manpage.push('\n');

    // Description
    manpage.push_str(".SH DESCRIPTION\n");
    manpage.push_str(&format_description(&main_help));
    manpage.push('\n');

    // Options
    if let Some(options) = extract_options(&main_help) {
        manpage.push_str(".SH OPTIONS\n");
        manpage.push_str(&options);
        manpage.push('\n');
    }

    // Subcommands
    if !subcommands.is_empty() {
        manpage.push_str(".SH SUBCOMMANDS\n");
        for subcmd in subcommands {
            if let Ok(subcmd_help) = get_command_output(binary_path, &[&subcmd, "--help"]) {
                manpage.push_str(&format!(
                    ".SS {}\n{}\n",
                    subcmd,
                    convert_subcommand_help(&subcmd_help),
                ));
            }
        }
    }


    // See Also Section (only if homepage or Git repo is available)
    let homepage = extract_homepage();
    let git_repo = extract_git_repo_url();
    if homepage.is_some() || git_repo.is_some() {
	manpage.push_str(".SH SEE ALSO\n");
        if let Some(homepage) = homepage {
            manpage.push_str(&format!("Homepage: {}\n\n", homepage));
        }
        if let Some(git_repo) = git_repo {
            manpage.push_str(&format!("Git repo: {}\n\n", git_repo));
        }
    }

    Ok(manpage)
}

/// Extracts the homepage URL from the `Cargo.toml` file, if available.
///
/// # Returns
/// * `Some(String)` containing the homepage URL if found.
/// * `None` if no homepage is specified or the file is unavailable.
///
/// This is a little extra functionality for rust projects.  If you use this program
/// on non-rust projects, it will just return None.
fn extract_homepage() -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let cargo_toml_path = cwd.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return None;
    }

    let mut file = File::open(cargo_toml_path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;

    // Parse the Cargo.toml file
    let cargo_toml: toml::Value = toml::from_str(&contents).ok()?;

    // First, check the `[package]` section for a homepage
    if let Some(homepage) = cargo_toml.get("package").and_then(|pkg| pkg.get("homepage")).and_then(|h| h.as_str()) {
        return Some(homepage.to_string());
    }

    // Next, check the `[workspace.package]` section for a homepage
    if let Some(homepage) = cargo_toml.get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|pkg| pkg.get("homepage"))
        .and_then(|h| h.as_str()) 
    {
        return Some(homepage.to_string());
    }

    None
}


/// Extracts the Git repository URL from the `.git/config` file, if available.
///
/// # Returns
/// * `Some(String)` containing the repository URL if found.
/// * `None` if no URL is found or the file is unavailable.
fn extract_git_repo_url() -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let git_config_path = cwd.join(".git/config");
    if !git_config_path.exists() {
        return None;
    }

    let file = File::open(git_config_path).ok()?;
    let reader = BufReader::new(file);

    let mut in_remote_origin = false;
    for line in reader.lines().flatten() {
        let trimmed = line.trim();
        if trimmed.starts_with("[remote \"origin\"]") {
            in_remote_origin = true;
        } else if in_remote_origin && trimmed.starts_with("url =") {
            // Extract the URL after "url ="
            if let Some(url) = trimmed.split('=').nth(1) {
                return Some(url.trim().to_string());
            }
        } else if trimmed.starts_with('[') {
            // Exit the "[remote \"origin\"]" section
            in_remote_origin = false;
        }
    }

    None
}

fn format_description(help_text: &str) -> String {
    let mut description = String::new();

    for line in help_text.lines() {
        if line.trim().starts_with("USAGE:") || line.trim().starts_with("OPTIONS:") {
            break;
        }
        if !line.trim().is_empty() {
            description.push_str(line.trim());
            description.push('\n');
        }
    }

    // Add line breaks for better rendering
    description
        .replace("-\n", "")
        .replace("-\r\n", "")
        .replace('`', "")
        .replace('*', "")
        .replace("\n", "\n\n") // Add spacing between paragraphs
}

/// Executes a command using the specified binary path and arguments, capturing its output.
///
/// # Arguments
/// * `binary_path` - Path to the binary to execute.
/// * `args` - A slice of string arguments to pass to the binary.
///
/// # Returns
/// * `Ok(String)` containing the captured standard output if the command executes successfully.
/// * `Err(String)` containing the captured standard error or an error message if the command fails.

fn get_command_output(binary_path: &PathBuf, args: &[&str]) -> Result<String, String> {
    let output = Command::new(binary_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Extracts the list of subcommands available in the binary based on its help output.
///
/// # Arguments
/// * `binary_path` - Path to the binary whose subcommands are to be extracted.
///
/// # Returns
/// * `Ok(Vec<String>)` containing the list of subcommands if successfully extracted.
/// * `Err(String)` containing an error message if the extraction fails.
fn get_subcommands(binary_path: &PathBuf) -> Result<Vec<String>, String> {
    let help_text = get_command_output(binary_path, &["--help"])?;
    let mut subcommands = Vec::new();
    let mut in_subcommands = false;

    for line in help_text.lines() {
        if line.trim().eq_ignore_ascii_case("SUBCOMMANDS:") || line.trim().eq_ignore_ascii_case("Commands:") {
            in_subcommands = true;
            continue;
        }
        if in_subcommands {
            if line.trim().is_empty() {
                break;
            }
            if let Some(subcmd) = line.split_whitespace().next() {
                if !subcmd.starts_with('-') && subcmd != "help" {
                    subcommands.push(subcmd.to_string());
                }
            }
        }
    }

    Ok(subcommands)
}

/// Extracts a short description from the binary's help output.
///
/// # Arguments
/// * `help_text` - The help output of the binary as a string.
///
/// # Returns
/// * A short description string, or a default string if none is found.
fn get_short_description(help_text: &str) -> String {
    help_text.lines()
        .find(|line| !line.trim().is_empty() && !line.trim().starts_with('U'))
        .map(|line| line.trim().to_string())
        .unwrap_or_else(|| "Command line tool".to_string())
}

/// Extracts the usage section from the binary's help output.
///
/// # Arguments
/// * `help_text` - The help output of the binary as a string.
///
/// # Returns
/// * `Some(String)` containing the usage section if found.
/// * `None` if no usage section is found.
fn get_usage(help_text: &str) -> Option<String> {
    help_text
        .lines()
        .find(|line| line.trim().starts_with("Usage:") || line.trim().starts_with("USAGE:"))
        .map(|line| {
            line.trim()
                .replacen("Usage:", "", 1)
                .replacen("USAGE:", "", 1)
                .trim()
                .to_string()
        })
}

/// Extracts the options section from the binary's help output.
///
/// # Arguments
/// * `help_text` - The help output of the binary as a string.
///
/// # Returns
/// * `Some(String)` containing the formatted options section if found.
/// * `None` if no options section is found.
fn extract_options(help_text: &str) -> Option<String> {
    let mut options = String::new();
    let mut in_options = false;

    for line in help_text.lines() {
        if line.trim().starts_with("OPTIONS:") {
            in_options = true;
            continue;
        }
        if in_options {
            if line.trim().is_empty() && !options.is_empty() {
                break;
            }
            if !line.trim().is_empty() {
                let formatted_line = line.replace("--", "\\-\\-").trim().to_string();
                options.push_str(&formatted_line);
                options.push('\n');
            }
        }
    }
    if options.is_empty() {
        None
    } else {
        Some(options)
    }
}

/// Converts the help output of a subcommand into a formatted manpage-compatible string.
///
/// # Arguments
/// * `help_text` - The help output of the subcommand as a string.
///
/// # Returns
/// * A formatted string suitable for inclusion in a manpage.
fn convert_subcommand_help(help_text: &str) -> String {
    help_text.replace("-\n", "").replace("-\r\n", "").replace("USAGE:", ".SH USAGE\n")
        .replace("OPTIONS:", ".SH OPTIONS\n").replace("--", "\\-\\-")
        .replace('`', "").replace('*', "")
}
