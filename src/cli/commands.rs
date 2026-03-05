//! CLI commands implementation

use std::io::{self, Read, Write};
use std::path::PathBuf;

use crate::core::{Profile, ProfileStatus, Vault};
use crate::error::Result;
use crate::parser;
use crate::vault;

/// Build the CLI parser
pub fn build_cli() -> clap::Command {
    use clap::{Arg, Command};
    
    Command::new("esimvault")
        .version("0.1.0")
        .author("Your Name")
        .about("Offline-first eSIM wallet/manager TUI tool")
        .arg(
            Arg::new("vault")
                .short('v')
                .long("vault")
                .value_name("PATH")
                .help("Vault file path")
                .env("ESIMVAULT_PATH")
                .global(true),
        )
        .arg(
            Arg::new("passphrase")
                .short('p')
                .long("passphrase")
                .value_name("PASSPHRASE")
                .help("Vault passphrase")
                .env("ESIMVAULT_PASSPHRASE")
                .global(true),
        )
        .arg(
            Arg::new("pass-stdin")
                .long("pass-stdin")
                .action(clap::ArgAction::SetTrue)
                .help("Read passphrase from stdin")
                .global(true),
        )
        .subcommand(
            Command::new("init")
                .about("Initialize a new vault")
                .arg(
                    Arg::new("vault")
                        .short('v')
                        .long("vault")
                        .value_name("PATH")
                        .help("Vault file path (default: ./vault.esimvault)"),
                ),
        )
        .subcommand(
            Command::new("add")
                .about("Add a new eSIM profile")
                .arg(
                    Arg::new("text")
                        .short('t')
                        .long("text")
                        .value_name("LPA_PAYLOAD")
                        .help("LPA payload text"),
                )
                .arg(
                    Arg::new("label")
                        .short('l')
                        .long("label")
                        .value_name("LABEL")
                        .help("Profile label"),
                )
                .arg(
                    Arg::new("tag")
                        .short('g')
                        .long("tag")
                        .value_name("TAG")
                        .help("Region tags (can be specified multiple times)")
                        .action(clap::ArgAction::Append),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("List all profiles")
                .arg(
                    Arg::new("status")
                        .short('s')
                        .long("status")
                        .value_name("STATUS")
                        .help("Filter by status (unused/used/expired)"),
                )
                .arg(
                    Arg::new("tag")
                        .short('t')
                        .long("tag")
                        .value_name("TAG")
                        .help("Filter by tag")
                        .action(clap::ArgAction::Append),
                )
                .arg(
                    Arg::new("search")
                        .short('f')
                        .long("search")
                        .value_name("QUERY")
                        .help("Search query"),
                ),
        )
        .subcommand(
            Command::new("show")
                .about("Show profile details")
                .arg(Arg::new("id").required(true).value_name("ID"))
                .arg(
                    Arg::new("reveal")
                        .long("reveal")
                        .help("Reveal sensitive data (shows warning)"),
                ),
        )
        .subcommand(
            Command::new("mark")
                .about("Mark profile status")
                .arg(Arg::new("id").required(true).value_name("ID"))
                .arg(Arg::new("unused").long("unused").help("Mark as unused"))
                .arg(Arg::new("used").long("used").help("Mark as used"))
                .arg(Arg::new("expired").long("expired").help("Mark as expired")),
        )
        .subcommand(
            Command::new("qr")
                .about("Generate QR code")
                .arg(Arg::new("id").required(true).value_name("ID"))
                .arg(
                    Arg::new("out")
                        .short('o')
                        .long("out")
                        .value_name("PATH")
                        .help("Output file path"),
                ),
        )
        .subcommand(
            Command::new("tui")
                .about("Start TUI")
                .arg(
                    Arg::new("vault")
                        .short('v')
                        .long("vault")
                        .value_name("PATH")
                        .help("Vault file path"),
                ),
        )
        .subcommand(
            Command::new("edit")
                .about("Edit profile")
                .arg(Arg::new("id").required(true).value_name("ID"))
                .arg(
                    Arg::new("label")
                        .long("label")
                        .value_name("LABEL")
                        .help("New label"),
                )
                .arg(
                    Arg::new("add-tag")
                        .long("add-tag")
                        .value_name("TAG")
                        .help("Add tag")
                        .action(clap::ArgAction::Append),
                )
                .arg(
                    Arg::new("remove-tag")
                        .long("remove-tag")
                        .value_name("TAG")
                        .help("Remove tag")
                        .action(clap::ArgAction::Append),
                )
                .arg(
                    Arg::new("notes")
                        .long("notes")
                        .value_name("NOTES")
                        .help("Set notes"),
                ),
        )
        .subcommand(
            Command::new("check-update")
                .about("Check for updates")
                .long_about("Check if a newer version is available on GitHub releases")
                .arg(
                    Arg::new("repo")
                        .short('r')
                        .long("repo")
                        .value_name("REPO")
                        .help("GitHub repository in format 'owner/repo' (default: Yeats33/esim-vault)"),
                ),
        )
        .subcommand(
            Command::new("version")
                .about("Show version information"),
        )
}

/// Get passphrase from CLI args or prompt
pub fn get_passphrase(matches: &clap::ArgMatches) -> Result<String> {
    // Check environment variable first
    if let Ok(passphrase) = std::env::var("ESIMVAULT_PASSPHRASE") {
        if !passphrase.is_empty() {
            return Ok(passphrase);
        }
    }
    
    // Check CLI argument
    if let Some(passphrase) = matches.get_one::<String>("passphrase") {
        return Ok(passphrase.clone());
    }
    
    // Check --pass-stdin
    if matches.get_flag("pass-stdin") {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        return Ok(input.trim().to_string());
    }
    
    // Prompt for passphrase
    print!("Enter vault passphrase: ");
    io::stdout().flush()?;
    
    // Try rpassword, fallback to basic read
    match rpassword::read_password() {
        Ok(pass) => Ok(pass),
        Err(_) => {
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;
            Ok(line.trim().to_string())
        }
    }
}

/// Get vault path from CLI args or default
pub fn get_vault_path(matches: &clap::ArgMatches) -> PathBuf {
    // Check CLI argument
    if let Some(vault) = matches.get_one::<String>("vault") {
        return PathBuf::from(vault);
    }
    
    // Check environment variable
    if let Ok(path) = std::env::var("ESIMVAULT_PATH") {
        return PathBuf::from(path);
    }
    
    // Default path
    PathBuf::from("vault.esimvault")
}

/// Run the CLI with parsed arguments
pub fn run_cli(matches: clap::ArgMatches) -> Result<()> {
    let vault_path = get_vault_path(&matches);
    let passphrase = get_passphrase(&matches)?;
    
    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            let path: PathBuf = sub_matches
                .get_one::<String>("vault")
                .map(|s| PathBuf::from(s))
                .unwrap_or_else(|| vault_path.clone());
            
            println!("Creating new vault at: {}", path.display());
            let _vault = vault::create_vault(&path, &passphrase)?;
            println!("Vault created successfully!");
            Ok(())
        }
        
        Some(("add", sub_matches)) => {
            // Load vault
            let mut vault = vault::load_vault(&vault_path, &passphrase)?;
            
            // Get LPA payload
            let text = sub_matches
                .get_one::<String>("text")
                .cloned()
                .unwrap_or_else(|| {
                    println!("Enter LPA payload:");
                    let mut input = String::new();
                    std::io::stdin().read_to_string(&mut input).unwrap();
                    input.trim().to_string()
                });
            
            // Validate LPA
            if let Err(e) = parser::parse_lpa(&text) {
                eprintln!("Warning: Failed to parse LPA payload: {}", e);
            }
            
            // Get label
            let label = sub_matches
                .get_one::<String>("label")
                .cloned()
                .unwrap_or_else(|| {
                    println!("Enter label (e.g., 'Airalo Japan 10GB'):");
                    let mut input = String::new();
                    std::io::stdin().read_to_string(&mut input).unwrap();
                    input.trim().to_string()
                });
            
            // Get tags
            let tags: Vec<String> = sub_matches
                .get_many::<String>("tag")
                .map(|v| v.cloned().collect())
                .unwrap_or_default();
            
            // Create profile
            let mut profile = Profile::new(label, text);
            for tag in tags {
                profile.add_tag(tag);
            }
            
            vault.add_profile(profile);
            
            // Save vault
            vault::save_vault(&vault, &vault_path, &passphrase)?;
            println!("Profile added successfully!");
            
            Ok(())
        }
        
        Some(("list", sub_matches)) => {
            let vault = vault::load_vault(&vault_path, &passphrase)?;
            
            // Parse filters
            let status_filter = sub_matches
                .get_one::<String>("status")
                .and_then(|s| s.parse::<ProfileStatus>().ok());
            
            let tag_filter: Vec<String> = sub_matches
                .get_many::<String>("tag")
                .map(|v| v.cloned().collect())
                .unwrap_or_default();
            
            let search_query = sub_matches
                .get_one::<String>("search")
                .cloned()
                .unwrap_or_default();
            
            // Filter profiles
            let profiles: Vec<&Profile> = vault.profiles.iter().filter(|p| {
                if let Some(status) = &status_filter {
                    if &p.status != status {
                        return false;
                    }
                }
                
                if !tag_filter.is_empty() {
                    if !tag_filter.iter().any(|t| p.region_tags.contains(t)) {
                        return false;
                    }
                }
                
                if !search_query.is_empty() {
                    let q = search_query.to_lowercase();
                    if !p.label.to_lowercase().contains(&q)
                        && !p.provider.as_ref().map(|s| s.to_lowercase()).unwrap_or_default().contains(&q)
                        && !p.region_tags.iter().any(|t| t.to_lowercase().contains(&q))
                    {
                        return false;
                    }
                }
                
                true
            }).collect();
            
            // Print table
            println!("\n{:36} | {:12} | {:15} | {}", "ID", "Status", "Provider", "Label");
            println!("{:-<36}-+-{:-<12}-+-{:-<15}-+-{:-<30}", '-', '-', '-', '-');
            
            let total = profiles.len();
            for p in &profiles {
                let status = match p.status {
                    ProfileStatus::Unused => "unused",
                    ProfileStatus::Used => "used",
                    ProfileStatus::Expired => "expired",
                };
                let provider = p.provider.as_deref().unwrap_or("-");
                println!("{} | {:12} | {:15} | {}", 
                    &p.id[..8], 
                    status, 
                    provider, 
                    p.label);
            }
            
            println!("\nTotal: {} profiles\n", total);
            
            Ok(())
        }
        
        Some(("show", sub_matches)) => {
            let vault = vault::load_vault(&vault_path, &passphrase)?;
            let id = sub_matches.get_one::<String>("id").unwrap();
            let reveal = sub_matches.get_flag("reveal");
            
            let profile = vault.get_profile(id)
                .ok_or_else(|| crate::error::Error::ProfileNotFound(id.clone()))?;
            
            println!("\n=== Profile Details ===\n");
            println!("ID:         {}", profile.id);
            println!("Label:      {}", profile.label);
            println!("Provider:   {}", profile.provider.as_deref().unwrap_or("-"));
            println!("Status:     {}", profile.status);
            
            println!("\nTags:");
            if profile.region_tags.is_empty() {
                println!("  (none)");
            } else {
                for tag in &profile.region_tags {
                    println!("  #{}", tag);
                }
            }
            
            println!("\nTimestamps:");
            println!("  Created: {}", profile.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!("  Updated: {}", profile.updated_at.format("%Y-%m-%d %H:%M:%S"));
            
            println!("\nLPA Payload:");
            if reveal {
                println!("  [REVEALED] {}", profile.lpa_payload_raw);
                eprintln!("\nWARNING: Sensitive data is now visible!\n");
            } else {
                println!("  [HIDDEN] (use --reveal to show, but be careful!)");
                println!("  Preview: {}...", &profile.lpa_payload_raw[..profile.lpa_payload_raw.len().min(30)]);
            }
            
            if let Some(parsed) = &profile.parsed {
                println!("\nParsed LPA:");
                if let Some(smdp) = &parsed.smdp {
                    let display = if reveal { smdp.clone() } else { mask(smdp) };
                    println!("  SM-DP+: {}", display);
                }
                if let Some(ac) = &parsed.activation_code {
                    let display = if reveal { ac.clone() } else { mask(ac) };
                    println!("  Activation: {}", display);
                }
                if let Some(cc) = &parsed.confirmation_code {
                    let display = if reveal { cc.clone() } else { mask(cc) };
                    println!("  Confirmation: {}", display);
                }
            }
            
            if let Some(notes) = &profile.notes {
                println!("\nNotes:\n{}", notes);
            }
            
            println!();
            Ok(())
        }
        
        Some(("mark", sub_matches)) => {
            let mut vault = vault::load_vault(&vault_path, &passphrase)?;
            let id = sub_matches.get_one::<String>("id").unwrap();
            
            let new_status = if sub_matches.get_flag("unused") {
                ProfileStatus::Unused
            } else if sub_matches.get_flag("used") {
                ProfileStatus::Used
            } else if sub_matches.get_flag("expired") {
                ProfileStatus::Expired
            } else {
                return Err(crate::error::Error::Vault("No status specified".to_string()));
            };
            
            let profile = vault.get_profile_mut(id)
                .ok_or_else(|| crate::error::Error::ProfileNotFound(id.clone()))?;
            
            let old_status = profile.status;
            profile.set_status(new_status);
            
            vault::save_vault(&vault, &vault_path, &passphrase)?;
            println!("Status changed from {} to {}", old_status, new_status);
            
            Ok(())
        }
        
        Some(("qr", sub_matches)) => {
            let vault = vault::load_vault(&vault_path, &passphrase)?;
            let id = sub_matches.get_one::<String>("id").unwrap();
            let out_path = sub_matches
                .get_one::<PathBuf>("out")
                .cloned()
                .unwrap_or_else(|| PathBuf::from(format!("{}.png", &id[..8])));
            
            let profile = vault.get_profile(id)
                .ok_or_else(|| crate::error::Error::ProfileNotFound(id.clone()))?;
            
            #[cfg(feature = "qr-encode")]
            {
                use std::fs;
                
                let qr_data = parser::generate_qr_image(&profile.lpa_payload_raw, 300)?;
                fs::write(&out_path, qr_data)?;
                println!("QR code saved to: {}", out_path.display());
                Ok(())
            }
            
            #[cfg(not(feature = "qr-encode"))]
            {
                Err(crate::error::Error::Qr("QR encoding not available. Compile with qr-encode feature.".to_string()))
            }
        }
        
        Some(("tui", _sub_matches)) => {
            // TUI is handled in main.rs
            Ok(())
        }
        
        Some(("edit", sub_matches)) => {
            let mut vault = vault::load_vault(&vault_path, &passphrase)?;
            let id = sub_matches.get_one::<String>("id").unwrap();
            
            let profile = vault.get_profile_mut(id)
                .ok_or_else(|| crate::error::Error::ProfileNotFound(id.clone()))?;
            
            if let Some(label) = sub_matches.get_one::<String>("label") {
                profile.set_label(label.clone());
                println!("Label updated to: {}", label);
            }
            
            if let Some(tags) = sub_matches.get_many::<String>("add-tag") {
                for tag in tags {
                    profile.add_tag(tag.clone());
                    println!("Added tag: #{}", tag);
                }
            }
            
            if let Some(tags) = sub_matches.get_many::<String>("remove-tag") {
                for tag in tags {
                    profile.remove_tag(tag);
                    println!("Removed tag: #{}", tag);
                }
            }
            
            if let Some(notes) = sub_matches.get_one::<String>("notes") {
                profile.set_notes(Some(notes.clone()));
                println!("Notes updated");
            }
            
            vault::save_vault(&vault, &vault_path, &passphrase)?;
            println!("Profile updated successfully!");
            
            Ok(())
        }
        
        Some(("check-update", sub_matches)) => {
            #[cfg(feature = "check-update")]
            {
                let repo = sub_matches
                    .get_one::<String>("repo")
                    .cloned()
                    .unwrap_or_else(|| "Yeats33/esim-vault".to_string());
                
                let parts: Vec<&str> = repo.split('/').collect();
                if parts.len() != 2 {
                    return Err(crate::error::Error::Vault("Invalid repository format. Use 'owner/repo'".to_string()));
                }
                
                let (owner, name) = (parts[0], parts[1]);
                
                println!("Checking for updates...");
                
                match crate::update::check_for_update(owner, name) {
                    Ok(status) => {
                        match status {
                            crate::update::UpdateStatus::UpToDate => {
                                println!("✓ You're running the latest version!");
                                println!("  Current version: {}", crate::update::get_current_version());
                            }
                            crate::update::UpdateStatus::UpdateAvailable(tag) => {
                                println!("✗ A new version is available!");
                                println!("  Current version: {}", crate::update::get_current_version());
                                println!("  Latest version:  {}", tag);
                                println!("\nVisit https://github.com/{}/releases to download the update.", repo);
                            }
                            crate::update::UpdateStatus::NoReleases => {
                                println!("ℹ No releases found for this repository.");
                                println!("  Current version: {}", crate::update::get_current_version());
                                println!("\nVisit https://github.com/{}/releases to check for updates.", repo);
                            }
                        }
                        Ok(())
                    }
                    Err(e) => {
                        // Check if it's a 404 (repository not found or no releases)
                        if e.contains("404") {
                            eprintln!("No releases found for {}", repo);
                            eprintln!("Please visit https://github.com/{}/releases to check for updates manually.", repo);
                        } else {
                            eprintln!("Error checking for updates: {}", e);
                        }
                        Err(crate::error::Error::Vault(e))
                    }
                }
            }
            
            #[cfg(not(feature = "check-update"))]
            {
                eprintln!("Update checking is not enabled.");
                eprintln!("Compile with --features check-update to enable this feature.");
                Err(crate::error::Error::Vault("Update checking not enabled".to_string()))
            }
        }
        
        Some(("version", _sub_matches)) => {
            println!("esim-vault {}", crate::update::get_current_version());
            Ok(())
        }
        
        _ => {
            // No subcommand - show help
            Err(crate::error::Error::Vault("No command specified. Use --help for usage.".to_string()))
        }
    }
}

fn mask(s: &str) -> String {
    if s.len() <= 4 {
        "*".repeat(s.len())
    } else {
        let visible = &s[..2];
        let end = &s[s.len() - 2..];
        format!("{}...{}", visible, end)
    }
}
