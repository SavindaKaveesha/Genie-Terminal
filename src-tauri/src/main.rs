#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod dictionary_module;

use std::process::{Command};
use std::path::PathBuf;

use std::collections::HashMap;
use tauri::Error;
use dictionary_module::Dictionary;

#[derive(serde::Serialize)]
struct CommandOutput {
    cwd: String,
    output: String,
}

#[tauri::command]
fn print_cmd_output(name: String, cwd: String) -> CommandOutput {
//    let cwd = r"C:\";
    let output = execute_command(name, &cwd);
    //let cwds = "c:\\";
    //format!("{}", output)
    CommandOutput { cwd, output }
}


#[tauri::command]
fn get_suggestions(name: &str) -> Result<HashMap<String, Vec<String>>, Error> {
    
    let mut dictionary = Dictionary::new("dictionary.db");

    dictionary.read_data().unwrap();

    let output = dictionary.find_pairs(name);

    return Ok(output);
}


fn execute_command(command: String, cwd: &str) -> String {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C").arg(&command);
    cmd.current_dir(PathBuf::from(&cwd));

    let output = match cmd.output() {
        Ok(output) => output,
        Err(e) => panic!("Failed to execute command: {}", e),
    };

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    format!("{}\n{}", stdout, stderr)
}


fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![print_cmd_output, get_suggestions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}