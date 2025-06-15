use inquire::{
    error::InquireResult,
    formatter::{MultiOptionFormatter, OptionFormatter},
    validator::Validation,
    InquireError, MultiSelect, Select, Text,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    env, fmt,
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    process::Command,
};

// The struct for a command
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
struct Cmd {
    command: String,
    description: String,
    tags: Vec<String>,
}

impl Cmd {
    fn new(command: String, description: String, tags: Vec<String>) -> Self {
        Self {
            command,
            description,
            tags,
        }
    }
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\n\t- desc: {}\n\t- tags: [{}]",
            self.command,
            self.description,
            self.tags.join(", ")
        )
    }
}

fn main() -> InquireResult<()> {
    // load in the commands
    let mut commands: Vec<Cmd> = load()?;

    // genarate a list of tags from the commands
    let tags: HashSet<String> = commands
        .iter()
        .flat_map(|cmd| cmd.tags.iter().map(|t| t.trim().to_string())) // Trim during cloning
        .collect();

    // Figure out what mode we are in
    let args: Vec<String> = env::args().collect();

    if args.is_empty() {
        return Err(InquireError::InvalidConfiguration(
            "You must supply an argument either:\n-a for adding a command\n-s for searching a saved command".to_string(),
        ));
    }

    let mode = args[1].trim();

    // Run a prompt
    match mode {
        "-s" => {
            let cmd = search_prompt(&commands, tags)?;
            // Run our searched command
            run_command(cmd)?;
            Ok(())
        }
        "-m" => {
            // Search
            let cmd = search_prompt(&commands, tags)?;
            // delete
            delete(&mut commands, cmd);
            // write the new-correct command
            add_promt(&mut commands)?;
            // Save commands
            save(&mut commands)?;
            Ok(())
        }
        "-d" => {
            // Search
            let cmd = search_prompt(&commands, tags)?;
            let cmd_cmd = cmd.command.clone();
            // Delete the selected command
            delete(&mut commands, cmd);
            println!("Deleted command: {}", cmd_cmd);
            // Save
            save(&mut commands)?;
            Ok(())
        }
        "-a" => {
            // Add the new command
            add_promt(&mut commands)?;
            // Save commands
            save(&mut commands)?;
            Ok(())
        }
        _ => {
            Err(InquireError::InvalidConfiguration(
                "Incorrect Flag used".to_string(),
            ))
        }
    }
}

fn search_prompt(commands: &[Cmd], tags: HashSet<String>) -> Result<Cmd, InquireError> {
    // Generate formatters
    let tag_formatter: MultiOptionFormatter<String> = &|a| {
        format!(
            "Selected tags: [{}]",
            a.iter()
                .map(|item| item.value.clone())
                .collect::<Vec<String>>()
                .join(", ")
        )
    };

    let cmd_formatter: OptionFormatter<Cmd> = &|a| format!("Selected Command: {}", a.value.command);

    // Select search tags
    let tgs = MultiSelect::new("Select relavent tags:", tags.into_iter().collect())
        .with_formatter(tag_formatter)
        .prompt()
        .unwrap();

    // Select command to run
    let cmd = Select::new("Select a command:", search(commands, tgs))
        .with_formatter(cmd_formatter)
        .prompt()
        .unwrap();

    Ok(cmd)
}

fn add_promt(cmds: &mut Vec<Cmd>) -> Result<(), InquireError> {
    let command_validator = |input: &str| match input.is_empty() {
        true => Ok(Validation::Invalid("You must provide a command".into())),
        false => Ok(Validation::Valid),
    };

    // Tag validator
    let tag_validator = |input: &str| match option_split(input) {
        Some(()) => Ok(Validation::Valid),
        None => Ok(Validation::Invalid("Tags must be comma-separated".into())),
    };

    // Start the prompt
    let command: String = Text::new("Enter command to save:")
        .with_validator(command_validator)
        .prompt()
        .unwrap();

    let description: String = Text::new("Enter command description:").prompt().unwrap();

    let tags: String = Text::new("Enter command tags:")
        .with_help_message("Each tag must be comma separated")
        .with_validator(tag_validator)
        .prompt()
        .unwrap();

    let tag_ls: Vec<String> = tags.split(",").map(|t| t.trim().to_owned()).collect();

    let new_cmd: Cmd = Cmd::new(command, description, tag_ls);

    // Add new command to list
    cmds.push(new_cmd);

    Ok(())
}

fn get_config_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(format!("{}/.cmds/commands.json", home))
}

fn load() -> Result<Vec<Cmd>, InquireError> {
    let path = get_config_path();

    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create file if it doesn't exist
    if !path.exists() {
        println!("File does not exist. Creating...");
        let mut file = File::create(&path)?;
        file.write_all(b"[]")?;
    }

    // Read the file content directly
    let json_content: String = fs::read_to_string(&path)?;

    // Load and parse JSON commands
    let cmds: Vec<Cmd> = serde_json::from_str(&json_content)
        .map_err(|e| InquireError::InvalidConfiguration(e.to_string()))?;

    Ok(cmds)
}

fn search(cmds: &[Cmd], search_tags: Vec<String>) -> Vec<Cmd> {
    if search_tags.is_empty() {
        cmds.to_vec()
    } else {
        cmds.iter()
            .filter(|cmd| cmd.tags.iter().any(|tag| search_tags.contains(tag)))
            .cloned()
            .collect()
    }
}

fn option_split(input: &str) -> Option<()> {
    let parts: Vec<&str> = input.split(',').collect();

    match parts.first() {
        Some(h) if h.contains(";") || h.contains("-") || h.contains("\t") => None,
        Some(_) => Some(()),
        None => None,
    }
}

fn save(cmds: &mut Vec<Cmd>) -> Result<(), InquireError> {
    let path = get_config_path();
    let json: String = serde_json::to_string_pretty(cmds)
        .map_err(|e| InquireError::InvalidConfiguration(e.to_string()))?;

    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create file if it doesn't exist
    if !path.exists() {
        println!("File does not exist. Creating & Saving...");
        let mut file = File::create(&path)?;
        file.write_all(json.as_bytes())?;

        return Ok(());
    }

    // Write the serialized json to file
    let mut file: File = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.write_all(json.as_bytes())?;

    println!("Successfully updated commands.");

    Ok(())
}

fn delete(cmds: &mut Vec<Cmd>, cmd: Cmd) {
    cmds.retain(|c| *c != cmd);
}

fn run_command(command: Cmd) -> Result<(), InquireError> {
    let cmd_args: Vec<&str> = command.command.split_whitespace().collect();

    if cmd_args.is_empty() {
        return Err(InquireError::InvalidConfiguration(
            "Error: Command is empty and cannot be executed.".into(),
        ));
    }

    let (cmd, args) = cmd_args.split_at(1);

    println!("Executing Command: {}...", cmd[0]);

    Command::new(cmd[0])
        .args(args)
        .spawn()
        .map_err(InquireError::IO)?;

    Ok(())
}
