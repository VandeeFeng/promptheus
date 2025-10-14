use anyhow::Result;

use crate::utils::interactive::prompt_input;

/// Parse variables from command string in format <param> or <param=default>
pub fn parse_command_variables(command: &str) -> Vec<(String, Option<String>)> {
    use regex::Regex;

    let re = Regex::new(r"<([^>=]+)(?:=([^>]*))?>").unwrap();
    let mut variables = Vec::new();

    for cap in re.captures_iter(command) {
        let name = cap.get(1).unwrap().as_str().to_string();
        let default = cap.get(2).map(|m| m.as_str().to_string());
        variables.push((name, default));
    }

    variables
}

/// Replace variables in command with provided values
pub fn replace_command_variables(
    command: &str,
    variables: &std::collections::HashMap<String, String>
) -> String {
    use regex::Regex;

    let re = Regex::new(r"<([^>=]+)(?:=([^>]*))?>").unwrap();

    re.replace_all(command, |caps: &regex::Captures| {
        let var_name = caps.get(1).unwrap().as_str();

        // Use provided value, or default, or empty string
        if let Some(value) = variables.get(var_name) {
            value.clone()
        } else if let Some(default_val) = caps.get(2) {
            default_val.as_str().to_string()
        } else {
            String::new()
        }
    }).to_string()
}

/// Prompt user for variable values interactively
pub fn prompt_for_variables(
    variables: Vec<(String, Option<String>)>
) -> Result<std::collections::HashMap<String, String>> {
    let mut result = std::collections::HashMap::new();

    for (name, default) in variables {
        let prompt = if let Some(ref default_val) = default {
            format!("{} [default: {}]: ", name, default_val)
        } else {
            format!("{}: ", name)
        };

        let input = prompt_input(&prompt)?;

        if input.is_empty() {
            if let Some(default_val) = default {
                result.insert(name, default_val);
            } else {
                result.insert(name, String::new());
            }
        } else {
            result.insert(name, input);
        }
    }

    Ok(result)
}

