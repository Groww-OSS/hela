use serde_json::Value;

use crate::utils::common::{execute_command, print_error};

pub struct SecretTool;

impl SecretTool {
    pub fn new() -> Self {
        SecretTool
    }

    pub async fn run_scan(&self, _path: &str, _commit_id: Option<&str>) {
        /*
            1. Clone Repo
            2. Get Commit ID to scan and checkout to that commit ID using git checkout <Commit-ID>
            3. Now copy only modified files from that commitID to another folder for scanning using git diff-tree --no-commit-id --name-only -r <Commit-ID> | xargs -I {} cp {} ~/Desktop/new_code/
        */
        // check if path is a local path ore git link and then clone it
        // if /tmp/app not exists then run below commands
        if !std::path::Path::new("/tmp/app").exists() {
            if _path.starts_with("http") {
                println!("Cloning git repo...");
                let clone_command = format!("git clone {} /tmp/app", _path);
                execute_command(&clone_command, true).await;
            }else{
                println!("Copying project to /tmp/app...");
                let copy_command = format!("cp -r {} /tmp/app", _path);
                execute_command(&copy_command, true).await;
            }
        }
        let mut _path = format!("/tmp/app");
        
        // if commit_id is provided then checkout to that commit id
        if let Some(commit_id) = _commit_id {
            println!("Checking out to commit id: {}", commit_id);
            let checkout_command = format!("cd {} && git checkout {}", _path, commit_id);
            execute_command(&checkout_command, true).await;
            // now copy only modified files from that commitID to new folder /tmp/new_code after creating new_code folder
            // make a new folder /tmp/new_code
            let copy_command = format!("mkdir -p /tmp/new_code");
            execute_command(&copy_command, true).await;
            let copy_command = format!("cd {} && git diff-tree --no-commit-id --name-only -r {} | xargs -I {{}} git ls-tree --name-only {} {{}} | xargs git archive --format=tar {} | tar -x -C /tmp/new_code", _path, commit_id, commit_id, commit_id);
            execute_command(&copy_command, true).await;
            // now run secret scan on /tmp/new_code folder
            _path = format!("/tmp/new_code");
        }

        println!("Running secret scan on path: {}", _path);

        let cmd = "trufflehog --version";
        let out = execute_command(cmd, true).await;
        if out == "" {
            print_error("Error: Secret Scanner is not configured properly, please contact support team!", 101);
        }
        // trufflehog filesystem --no-update /tmp/app --json >
        let cmd = format!("trufflehog filesystem --no-update {} --json", _path);
        let output_data = execute_command(&cmd, true).await;
        let mut results: Vec<Value> = Vec::new();
        for line in output_data.lines() {
            let json_output: serde_json::Value = serde_json::from_str(&line).expect("Error parsing JSON");
            // if it have key SourceMetadata only then add it to results
            if json_output["SourceMetadata"].is_null() {
                continue;
            }
            results.push(json_output);
        }
        let json_output = serde_json::json!({
            "results": results
        });
        let json_output = serde_json::to_string_pretty(&json_output).unwrap();
        std::fs::write("/tmp/secrets.json", json_output).expect("Unable to write file");
        let is_file_exists = std::path::Path::new("/tmp/secrets.json").exists();
        if !is_file_exists {
            print_error("Error: Secret Scanner not generated results, please contact support team!", 101);
        }
        let json_output = std::fs::read_to_string("/tmp/secrets.json").expect("Error reading file");
        let json_output: serde_json::Value = serde_json::from_str(&json_output).expect("Error parsing JSON");
        println!("{:?}", json_output);
    }
}
