use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::fs::File;
use std::io::{Read, BufReader, BufRead};

pub fn generate_manpage(binary_path: &PathBuf, name: &str, output_dir: &PathBuf, section: u8, title: &str) -> Result<(), String> {
    let manpage_path = output_dir.join(format!("{}.{}", name, section));

    // Generate manpage content
    let manpage_content = generate_manpage_content(binary_path, name, section, title)?;

    // Write manpage
    fs::write(&manpage_path, manpage_content)
        .map_err(|e| format!("Failed to write manpage: {}", e))?;

    println!("Manpage generated at {}", manpage_path.display());
    Ok(())
}

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

/// This function extracts the homepage URL from the Cargo.toml file, if one exists.
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

fn get_short_description(help_text: &str) -> String {
    help_text.lines()
        .find(|line| !line.trim().is_empty() && !line.trim().starts_with('U'))
        .map(|line| line.trim().to_string())
        .unwrap_or_else(|| "Command line tool".to_string())
}

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

fn convert_subcommand_help(help_text: &str) -> String {
    help_text.replace("-\n", "").replace("-\r\n", "").replace("USAGE:", ".SH USAGE\n")
        .replace("OPTIONS:", ".SH OPTIONS\n").replace("--", "\\-\\-")
        .replace('`', "").replace('*', "")
}
