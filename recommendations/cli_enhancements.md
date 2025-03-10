# ICN Command Line Interface Enhancements

Based on the analysis of the current CLI implementation, here are specific recommendations to enhance the ICN Command Line Interface for a more robust and user-friendly experience.

## Current State

The ICN CLI appears to be the most mature component of the system, with implementations for the following features:
- Health checks
- Identity management
- Cooperative management
- Resource management
- Governance operations
- Network management

## High-Priority Enhancements

### 1. Complete Command Implementation

```rust
// Add implementation for any missing commands
// Example enhancement for governance commands in src/commands/governance.rs

pub fn execute_governance_command(sub_matches: &ArgMatches) -> Result<(), CliError> {
    match sub_matches.subcommand() {
        Some(("propose", args)) => execute_propose_command(args),
        Some(("vote", args)) => execute_vote_command(args),
        Some(("list", args)) => execute_list_proposals_command(args),
        Some(("show", args)) => execute_show_proposal_command(args),
        Some(("execute", args)) => execute_execute_proposal_command(args), // New command
        Some(("cancel", args)) => execute_cancel_proposal_command(args),   // New command
        Some(("history", args)) => execute_proposal_history_command(args), // New command
        _ => {
            println!("Unknown governance command. Run 'icn governance --help' for usage information.");
            Ok(())
        }
    }
}

// Implement the new commands
fn execute_execute_proposal_command(args: &ArgMatches) -> Result<(), CliError> {
    let proposal_id = args.get_one::<String>("proposal-id").unwrap();
    let client = IcnClient::new(get_api_url(args)?)?;
    
    // Call the API to execute the proposal
    match client.execute_proposal(proposal_id).await {
        Ok(_) => {
            println!("✅ Proposal {} has been executed successfully.", proposal_id);
            Ok(())
        },
        Err(e) => Err(CliError::ApiError(format!("Failed to execute proposal: {}", e))),
    }
}
```

### 2. Improve Error Handling and Reporting

```rust
// Enhance the error types in src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Input validation error: {0}")]
    ValidationError(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

// Implement better error handling in commands
pub fn execute_command() -> Result<(), CliError> {
    // Command logic
    match api_call() {
        Ok(response) => {
            // Process response
            Ok(())
        },
        Err(e) => match e {
            ApiError::Unauthorized(_) => Err(CliError::AuthError("Session expired. Please log in again.".to_string())),
            ApiError::NotFound(_) => Err(CliError::ApiError("Resource not found. Check your input and try again.".to_string())),
            ApiError::ServerError(status, msg) => Err(CliError::ApiError(format!("Server error ({}): {}", status, msg))),
            ApiError::NetworkError(_) => Err(CliError::NetworkError("Could not connect to the ICN API. Check your network connection.".to_string())),
            _ => Err(CliError::UnknownError(format!("An unexpected error occurred: {}", e))),
        },
    }
}
```

### 3. Add Output Formatting Options

```rust
// Add a formatting module in src/output.rs
pub enum OutputFormat {
    Text,
    Json,
    Csv,
    Table,
}

pub fn format_output<T: Serialize>(data: &T, format: OutputFormat) -> Result<String, CliError> {
    match format {
        OutputFormat::Text => format_as_text(data),
        OutputFormat::Json => format_as_json(data),
        OutputFormat::Csv => format_as_csv(data),
        OutputFormat::Table => format_as_table(data),
    }
}

fn format_as_json<T: Serialize>(data: &T) -> Result<String, CliError> {
    serde_json::to_string_pretty(data)
        .map_err(|e| CliError::SystemError(format!("Failed to format output as JSON: {}", e)))
}

fn format_as_csv<T: Serialize>(data: &T) -> Result<String, CliError> {
    // Implementation for CSV formatting
    // ...
}

fn format_as_table<T: Serialize>(data: &T) -> Result<String, CliError> {
    // Implementation for table formatting using prettytable-rs
    // ...
}

// In command execution, use the formatter
fn execute_list_command(args: &ArgMatches) -> Result<(), CliError> {
    let format = match args.get_one::<String>("output") {
        Some(f) => match f.as_str() {
            "json" => OutputFormat::Json,
            "csv" => OutputFormat::Csv,
            "table" => OutputFormat::Table,
            _ => OutputFormat::Text,
        },
        None => OutputFormat::Text,
    };
    
    let client = IcnClient::new(get_api_url(args)?)?;
    let data = client.get_list().await?;
    
    let formatted_output = format_output(&data, format)?;
    println!("{}", formatted_output);
    
    Ok(())
}
```

### 4. Implement Configuration Profiles

```rust
// Add configuration profile support in src/config.rs
pub struct CliConfig {
    pub api_url: String,
    pub timeout: u64,
    pub default_format: String,
    pub verbose: bool,
    pub profile: String,
}

pub fn load_config(profile: Option<&str>) -> Result<CliConfig, CliError> {
    let profile_name = profile.unwrap_or("default");
    let config_path = get_config_path()?;
    
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| CliError::ConfigError(format!("Failed to read config file: {}", e)))?;
    
    let config: toml::Value = toml::from_str(&config_content)
        .map_err(|e| CliError::ConfigError(format!("Failed to parse config file: {}", e)))?;
    
    // Extract profile-specific settings
    let profile_config = config.get("profiles")
        .and_then(|p| p.get(profile_name))
        .ok_or_else(|| CliError::ConfigError(format!("Profile '{}' not found in config", profile_name)))?;
    
    // Build the config
    let api_url = profile_config.get("api_url")
        .and_then(|v| v.as_str())
        .unwrap_or("http://localhost:8082")
        .to_string();
    
    let timeout = profile_config.get("timeout")
        .and_then(|v| v.as_integer())
        .unwrap_or(30) as u64;
    
    let default_format = profile_config.get("default_format")
        .and_then(|v| v.as_str())
        .unwrap_or("text")
        .to_string();
    
    let verbose = profile_config.get("verbose")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    Ok(CliConfig {
        api_url,
        timeout,
        default_format,
        verbose,
        profile: profile_name.to_string(),
    })
}

// Update the config with a new profile
pub fn update_config_profile(profile: &str, key: &str, value: &str) -> Result<(), CliError> {
    let config_path = get_config_path()?;
    
    // Read existing config
    let mut config: toml::Value = if std::path::Path::new(&config_path).exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| CliError::ConfigError(format!("Failed to read config file: {}", e)))?;
        
        toml::from_str(&content)
            .map_err(|e| CliError::ConfigError(format!("Failed to parse config file: {}", e)))?
    } else {
        toml::Value::Table(toml::value::Table::new())
    };
    
    // Ensure profiles table exists
    if !config.as_table().unwrap().contains_key("profiles") {
        config.as_table_mut().unwrap().insert("profiles".to_string(), toml::Value::Table(toml::value::Table::new()));
    }
    
    // Ensure profile exists
    let profiles = config.get_mut("profiles").unwrap().as_table_mut().unwrap();
    if !profiles.contains_key(profile) {
        profiles.insert(profile.to_string(), toml::Value::Table(toml::value::Table::new()));
    }
    
    // Update the value
    let profile_table = profiles.get_mut(profile).unwrap().as_table_mut().unwrap();
    
    match key {
        "api_url" | "default_format" => {
            profile_table.insert(key.to_string(), toml::Value::String(value.to_string()));
        },
        "timeout" => {
            if let Ok(timeout) = value.parse::<i64>() {
                profile_table.insert(key.to_string(), toml::Value::Integer(timeout));
            } else {
                return Err(CliError::ValidationError("Timeout must be a number".to_string()));
            }
        },
        "verbose" => {
            if let Ok(verbose) = value.parse::<bool>() {
                profile_table.insert(key.to_string(), toml::Value::Boolean(verbose));
            } else {
                return Err(CliError::ValidationError("Verbose must be true or false".to_string()));
            }
        },
        _ => return Err(CliError::ValidationError(format!("Unknown configuration key: {}", key))),
    }
    
    // Write the updated config
    let config_str = toml::to_string(&config)
        .map_err(|e| CliError::ConfigError(format!("Failed to serialize config: {}", e)))?;
    
    std::fs::write(&config_path, config_str)
        .map_err(|e| CliError::ConfigError(format!("Failed to write config file: {}", e)))?;
    
    Ok(())
}
```

### 5. Add Interactive Mode

```rust
// Add interactive mode support in src/interactive.rs
use dialoguer::{theme::ColorfulTheme, Select, Input, Confirm};

pub fn run_interactive_mode() -> Result<(), CliError> {
    println!("Welcome to ICN Interactive Mode!");
    println!("================================");
    
    let theme = ColorfulTheme::default();
    
    loop {
        let categories = vec!["Identity", "Cooperative", "Governance", "Resources", "Network", "Exit"];
        
        let selection = Select::with_theme(&theme)
            .with_prompt("Select a category")
            .default(0)
            .items(&categories)
            .interact()
            .map_err(|e| CliError::SystemError(format!("Interactive mode error: {}", e)))?;
        
        match categories[selection] {
            "Identity" => handle_identity_interactive(&theme)?,
            "Cooperative" => handle_cooperative_interactive(&theme)?,
            "Governance" => handle_governance_interactive(&theme)?,
            "Resources" => handle_resources_interactive(&theme)?,
            "Network" => handle_network_interactive(&theme)?,
            "Exit" => break,
            _ => unreachable!(),
        }
    }
    
    println!("Exiting ICN Interactive Mode. Goodbye!");
    Ok(())
}

fn handle_identity_interactive(theme: &ColorfulTheme) -> Result<(), CliError> {
    let actions = vec!["Create Identity", "List Identities", "Show Identity", "Back"];
    
    let selection = Select::with_theme(theme)
        .with_prompt("Select an action")
        .default(0)
        .items(&actions)
        .interact()
        .map_err(|e| CliError::SystemError(format!("Interactive mode error: {}", e)))?;
    
    match actions[selection] {
        "Create Identity" => {
            println!("Creating a new identity...");
            let client = IcnClient::new(get_default_api_url()?)?;
            match client.create_identity().await {
                Ok(identity) => println!("✅ Identity created: {}", identity.did),
                Err(e) => println!("❌ Failed to create identity: {}", e),
            }
        },
        "List Identities" => {
            println!("Listing identities...");
            let client = IcnClient::new(get_default_api_url()?)?;
            match client.list_identities().await {
                Ok(identities) => {
                    if identities.is_empty() {
                        println!("No identities found.");
                    } else {
                        println!("Found {} identities:", identities.len());
                        for (i, identity) in identities.iter().enumerate() {
                            println!("{}. {}", i+1, identity.did);
                        }
                    }
                },
                Err(e) => println!("❌ Failed to list identities: {}", e),
            }
        },
        "Show Identity" => {
            let did = Input::<String>::with_theme(theme)
                .with_prompt("Enter DID")
                .interact()
                .map_err(|e| CliError::SystemError(format!("Interactive mode error: {}", e)))?;
            
            println!("Fetching identity details for {}...", did);
            let client = IcnClient::new(get_default_api_url()?)?;
            match client.get_identity(&did).await {
                Ok(identity) => {
                    println!("Identity Details:");
                    println!("DID: {}", identity.did);
                    println!("Created: {}", identity.created_at);
                    // Display other identity details
                },
                Err(e) => println!("❌ Failed to get identity: {}", e),
            }
        },
        "Back" => return Ok(()),
        _ => unreachable!(),
    }
    
    Ok(())
}

// Implement other interactive handlers (cooperative, governance, etc.)
```

### 6. Add Batch Operations Support

```rust
// Add batch operation support in src/batch.rs
pub fn execute_batch_command(args: &ArgMatches) -> Result<(), CliError> {
    let batch_file = args.get_one::<String>("file").unwrap();
    
    let batch_content = std::fs::read_to_string(batch_file)
        .map_err(|e| CliError::SystemError(format!("Failed to read batch file: {}", e)))?;
    
    let batch_commands: Vec<String> = batch_content.lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
        .map(|line| line.trim().to_string())
        .collect();
    
    println!("Executing batch with {} commands", batch_commands.len());
    
    let mut success_count = 0;
    let mut error_count = 0;
    
    for (i, cmd) in batch_commands.iter().enumerate() {
        println!("\nCommand {}/{}: {}", i+1, batch_commands.len(), cmd);
        
        let args_vec = shell_words::split(cmd)
            .map_err(|e| CliError::ValidationError(format!("Failed to parse command: {}", e)))?;
        
        let result = execute_cli_command(args_vec);
        
        match result {
            Ok(_) => {
                println!("✅ Command succeeded");
                success_count += 1;
            },
            Err(e) => {
                println!("❌ Command failed: {}", e);
                error_count += 1;
                
                if args.get_flag("stop-on-error") {
                    return Err(CliError::ValidationError(
                        format!("Batch execution stopped due to error in command {}: {}", i+1, cmd)
                    ));
                }
            }
        }
    }
    
    println!("\nBatch execution completed");
    println!("Successful commands: {}", success_count);
    println!("Failed commands: {}", error_count);
    
    Ok(())
}

// Function to execute a CLI command programmatically
fn execute_cli_command(args: Vec<String>) -> Result<(), CliError> {
    // Create a new CLI app instance
    let app = build_cli();
    
    // Parse the arguments
    let matches = app.try_get_matches_from(args)
        .map_err(|e| CliError::ValidationError(format!("Invalid command: {}", e)))?;
    
    // Execute the command
    match matches.subcommand() {
        Some(("identity", sub_m)) => execute_identity_command(sub_m),
        Some(("cooperative", sub_m)) => execute_cooperative_command(sub_m),
        Some(("governance", sub_m)) => execute_governance_command(sub_m),
        Some(("resource", sub_m)) => execute_resource_command(sub_m),
        Some(("network", sub_m)) => execute_network_command(sub_m),
        Some(("health", sub_m)) => execute_health_command(sub_m),
        Some(("config", sub_m)) => execute_config_command(sub_m),
        Some(("batch", _)) => Err(CliError::ValidationError("Nested batch commands are not allowed".to_string())),
        _ => Err(CliError::ValidationError("Unknown command".to_string())),
    }
}
```

### 7. Implement Shell Completion

```rust
// Add shell completion generator in src/completion.rs
pub fn generate_completion(shell: &str, out_dir: &str) -> Result<(), CliError> {
    use clap_complete::{generate_to, shells};
    
    let mut app = build_cli();
    let bin_name = "icn";
    
    match shell.to_lowercase().as_str() {
        "bash" => {
            generate_to(
                shells::Bash,
                &mut app,
                bin_name,
                out_dir,
            )
            .map_err(|e| CliError::SystemError(format!("Failed to generate completion: {}", e)))?;
            
            println!("Bash completion file generated at {}/{}.bash", out_dir, bin_name);
        },
        "zsh" => {
            generate_to(
                shells::Zsh,
                &mut app,
                bin_name,
                out_dir,
            )
            .map_err(|e| CliError::SystemError(format!("Failed to generate completion: {}", e)))?;
            
            println!("Zsh completion file generated at {}/_{}", out_dir, bin_name);
        },
        "fish" => {
            generate_to(
                shells::Fish,
                &mut app,
                bin_name,
                out_dir,
            )
            .map_err(|e| CliError::SystemError(format!("Failed to generate completion: {}", e)))?;
            
            println!("Fish completion file generated at {}/{}.fish", out_dir, bin_name);
        },
        "powershell" => {
            generate_to(
                shells::PowerShell,
                &mut app,
                bin_name,
                out_dir,
            )
            .map_err(|e| CliError::SystemError(format!("Failed to generate completion: {}", e)))?;
            
            println!("PowerShell completion file generated at {}/_{}.ps1", out_dir, bin_name);
        },
        _ => {
            return Err(CliError::ValidationError(format!("Unsupported shell: {}. Supported shells: bash, zsh, fish, powershell", shell)));
        }
    }
    
    Ok(())
}
```

### 8. Add Progress Indicators for Long-Running Operations

```rust
// Add progress indicators in src/progress.rs
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct ProgressIndicator {
    progress_bar: ProgressBar,
}

impl ProgressIndicator {
    pub fn new_spinner(message: &str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        
        ProgressIndicator { progress_bar: pb }
    }
    
    pub fn new_progress(len: u64, message: &str) -> Self {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        pb.set_message(message.to_string());
        
        ProgressIndicator { progress_bar: pb }
    }
    
    pub fn increment(&self, delta: u64) {
        self.progress_bar.inc(delta);
    }
    
    pub fn set_message(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
    }
    
    pub fn finish_with_message(&self, message: &str) {
        self.progress_bar.finish_with_message(message.to_string());
    }
}

// Example usage in a command
fn execute_network_diagnostics_command(args: &ArgMatches) -> Result<(), CliError> {
    let comprehensive = args.get_flag("comprehensive");
    
    println!("Running network diagnostics...");
    
    let progress = ProgressIndicator::new_spinner("Initializing diagnostics...");
    
    // Step 1: Check API connectivity
    progress.set_message("Checking API connectivity...");
    std::thread::sleep(Duration::from_secs(1)); // Simulating work
    
    // Step 2: Check peer connections
    progress.set_message("Checking peer connections...");
    std::thread::sleep(Duration::from_secs(2)); // Simulating work
    
    // Step 3: Measure latency
    progress.set_message("Measuring network latency...");
    std::thread::sleep(Duration::from_secs(1)); // Simulating work
    
    if comprehensive {
        // More detailed checks
        progress.set_message("Running comprehensive checks...");
        std::thread::sleep(Duration::from_secs(3)); // Simulating work
    }
    
    progress.finish_with_message("Diagnostics completed!");
    
    // Display results
    println!("\nDiagnostics Results:");
    println!("--------------------");
    println!("API connectivity: ✅ OK (23ms)");
    println!("Peer connections: ✅ OK (5/5 peers connected)");
    println!("Network latency: ✅ OK (avg: 45ms)");
    
    if comprehensive {
        println!("Bandwidth: ✅ OK (tx: 2.3 MB/s, rx: 1.8 MB/s)");
        println!("Packet loss: ✅ OK (0.2%)");
        println!("DNS resolution: ✅ OK (15ms)");
    }
    
    Ok(())
}
```

## Implementation Plan

1. **Phase 1: Core Improvements**
   - Enhance error handling
   - Add output formatting options
   - Implement configuration profiles

2. **Phase 2: User Experience Enhancements**
   - Add interactive mode
   - Implement progress indicators
   - Add command autocompletion

3. **Phase 3: Advanced Features**
   - Implement batch operations
   - Add advanced network diagnostics
   - Create scripting capabilities

## Testing Checklist

- [ ] Unit tests for all command handlers
- [ ] Unit tests for error handling
- [ ] Integration tests with mock API
- [ ] End-to-end tests with real API
- [ ] Test output formatting for different formats
- [ ] Test batch operations with various scenarios
- [ ] Test configuration profile management

## Documentation Updates

- [ ] Update help text for all commands
- [ ] Create examples for common operations
- [ ] Document configuration file format
- [ ] Create a user guide for interactive mode
- [ ] Document batch file format and examples

These enhancements will significantly improve the usability and reliability of the ICN CLI, making it a more effective tool for interacting with the ICN platform. 