use crate::game_data::parser::issue_handler::IssueHandler;
use log::info;
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Data Objects which should be ignored
const REJECTED_DATA_FILE_NAMES: [&str; 8] = [
    "Any Move.json",
    "Potion.json",
    "Super Potion.json",
    "Hyper Potion.json",
    "Max Potion.json",
    "Full Heal.json",
    "Full Restore.json",
    "White Herbs.json",
];

pub fn parse_file<T: DeserializeOwned>(
    file_path: PathBuf,
) -> Result<T, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)?;

    let result: T = serde_json::from_str(&json_data)?;
    Ok(result)
}

pub fn parse_directory<T: DeserializeOwned, I: IssueHandler>(
    base_path: &Path,
    subfolder: &str,
    parsing_issues: &mut I,
) -> Vec<T> {
    let mut result = Vec::new();

    let path = Path::new(base_path).join(subfolder);

    let Ok(entries) = std::fs::read_dir(path) else {
        return result;
    };

    for entry in entries.flatten() {
        if REJECTED_DATA_FILE_NAMES
            .iter()
            .any(|x| entry.path().ends_with(x))
        {
            info!("Skipping {:?}", entry.path());
            continue;
        }

        let file_path = entry.path();

        if file_path.is_file() && file_path.extension().map_or(false, |ext| ext == "json") {
            match parse_file::<T>(file_path) {
                Ok(parsed) => result.push(parsed),
                Err(err) => {
                    let file_name = entry.file_name().into_string().unwrap();

                    parsing_issues.handle_issue(format!(
                        "Failed to parse file `{subfolder}/{file_name}`: {}",
                        err,
                    ));
                }
            }
        }
    }

    result
}
