use std::collections::HashMap;

use serde_json::{json, Value};

use crate::utils::{common::{execute_command, post_json_data}, file_utils::find_files_recursively};

pub struct ScaTool;

pub static mut SUPPORTED_MANIFESTS : [&str; 4] = [
    "requirements.txt",
    "package-lock.json",
    "pom.xml",
    "pnpm-lock.yaml"
];


pub static mut DETECT_MANIFESTS : [&str; 4] = [
    "requirements.txt",
    "package.json",
    "pom.xml",
    "pnpm-lock.yaml"
];

impl ScaTool {
    pub fn new() -> Self {
        ScaTool
    }

    async fn install_project_dependencies(&self, _path: &str, ignore_dirs: Vec<&str>, detect_manifests: Vec<&str>, _no_install: bool, root_only: bool, build_args:String, verbose: bool) {
        // detect if project is python or nodejs based on manifest from DETECT_MANIFESTS and install that
        // installation script for each language would be in /tmp/install/ folder with file like python.sh, javascript.sh, java.sh etc for each language, developer need to write that script and we will execute it here based on language detection
        let installation_script_path = format!("{}/install", std::env::temp_dir().to_str().unwrap());
        let mut detected_files = Vec::new();
        if !root_only {
            detected_files = find_files_recursively(_path, detect_manifests, ignore_dirs).await;
        }else{
            for manifest in detect_manifests.iter() {
                let manifest_path = format!("{}/{}", _path, manifest);
                if std::path::Path::new(&manifest_path).exists() {
                    detected_files.push(manifest_path);
                }
            }
        }
        let language_mapping = {
            let mut map = std::collections::HashMap::new();
            map.insert("requirements.txt", "python");
            map.insert("package.json", "javascript");
            map.insert("package-lock.json", "javascript");
            map.insert("pom.xml", "maven");
            map.insert("build.gradle", "gradle");
            map.insert("go.mod", "go");
            map.insert("pnpm-lock.yaml", "pnpm-javascript");
            map
        };
        // check if we have one of manifest file from DETECT_MANIFESTS then install dependencies based on language_mapping
        if detected_files.len() > 0 {
           for detected_file in detected_files.iter() {
             let file_name = detected_file.split("/").last().unwrap();
             let folder_path = detected_file.replace(file_name, "");
             let language = language_mapping.get(file_name).unwrap().to_string();
             if language == "python" {
                if verbose {
                    println!("[+] Found python manifest file, installing python dependencies for {}", file_name);
                }
                let install_command = format!("cd {} && pip install -r {} {}", folder_path, file_name, build_args.clone());
                execute_command(&install_command, true).await;
                // check if installation script exists for python and then execute it
                if std::path::Path::new(&format!("{}/python.sh", installation_script_path)).exists() {
                    if verbose {
                        println!("[INFO] Found python installation script, executing it...");
                    }
                    let install_command = format!("cd {} && sh {}/python.sh", folder_path, installation_script_path);
                    execute_command(&install_command, true).await;
                }
                if verbose {
                    println!("[+] Installation of python dependencies completed!");
                }
             }
             if language == "javascript" || language == "pnpm-javascript" {
                // check if installation script exists for python and then execute it
                 if std::path::Path::new(&format!("{}/javascript.sh", installation_script_path)).exists() {
                    if verbose {
                        println!("[INFO] Found javascript installation script, executing it...");
                    }
                    let install_command = format!("cd {} && sh {}/javascript.sh", folder_path, installation_script_path);
                    execute_command(&install_command, true).await;
                 }
                 if verbose {
                    println!("[+] Installing javascript dependencies for {}...", file_name);
                 }
                 if language == "pnpm-javascript" {
                    let install_command = format!("cd {} && pnpm install --force --ignore-scripts {}", folder_path, build_args.clone());
                    println!("[INFO] Running command: {}", install_command);
                    execute_command(&install_command, true).await;
                 }else{
                    let install_command = format!("cd {} && npm install --force --ignore-scripts {}", folder_path, build_args.clone());
                    println!("[INFO] Running command: {}", install_command);
                    execute_command(&install_command, true).await;
                }
                if verbose {
                    println!("[+] Installation of javascript dependencies completed!");
                }
             }

             if language == "maven" {
                // check if installation script exists for python and then execute it
                 if std::path::Path::new(&format!("{}/java.sh", installation_script_path)).exists() {
                    if verbose {
                        println!("[INFO] Found java installation script, executing it...");
                    }
                    let install_command = format!("cd {} && sh {}/java.sh", folder_path, installation_script_path);
                    execute_command(&install_command, true).await;
                 }
                if verbose {
                    println!("[+] Installing maven dependencies for {}...", file_name);
                }
                 let install_command = format!("cd {} && mvn install {}", folder_path, build_args.clone());
                 execute_command(&install_command, true).await;
                if verbose {
                    println!("[+] Installation of maven dependencies completed!");
                }
             }

             if language == "gradle" {
                // check if installation script exists for python and then execute it
                 if std::path::Path::new(&format!("{}/java.sh", installation_script_path)).exists() {
                    if verbose {
                        println!("[INFO] Found java installation script, executing it...");
                    }
                    let install_command = format!("cd {} && sh {}/java.sh", folder_path, installation_script_path);
                    execute_command(&install_command, true).await;
                 }
                if verbose {
                    println!("[+] Installing gradle dependencies for {}...", file_name);
                }
                let install_command = format!("cd {} && gradle build {}", folder_path, build_args.clone());
                execute_command(&install_command, true).await;
                if verbose {
                    println!("[+] Installation of gradle dependencies completed!");
                }
             }
           }
        }
    }

    pub async fn run_scan(&self, _path: &str, _commit_id: Option<&str>, _branch: Option<&str>, _server_url: Option<&str>, no_install: bool, root_only: bool, build_args: String, manfiests: String, verbose: bool) {
        if verbose {
            println!("[+] Running SCA scan on path: {}", _path);
            println!("[+] Commit ID: {}", _commit_id.unwrap_or("None"));
            println!("[+] Branch: {}", _branch.unwrap_or("None"));
            println!("[+] Build args: {}", build_args.clone());
            println!("[+] Manifests: {}", manfiests.clone());
        }

        let mut new_manifests = Vec::new();
        let mut new_detect_manifests = Vec::new();

        if manfiests != "" {
            new_manifests = manfiests.split(",").collect::<Vec<&str>>();
            new_detect_manifests = manfiests.split(",").collect::<Vec<&str>>();
        }else{
            unsafe {
                new_manifests = SUPPORTED_MANIFESTS.to_vec();
                new_detect_manifests = DETECT_MANIFESTS.to_vec();
            }
        }

        let mut ignore_dirs = Vec::new();
        ignore_dirs.push("node_modules");
        ignore_dirs.push("bin");
        ignore_dirs.push("venv");
        ignore_dirs.push(".venv");

        if !std::path::Path::new("/tmp/app").exists() {
            if _path.starts_with("http") {
                if verbose {
                    println!("[+] Cloning git repo...");
                }
                if let Some(_branch) = _branch {
                    let clone_command = format!("git clone -b {} {} /tmp/app", _branch, _path);
                    execute_command(&clone_command, true).await;
                }else{
                    let clone_command = format!("git clone {} /tmp/app", _path);
                    execute_command(&clone_command, true).await;
                }
            }else{
                if verbose {
                    println!("[+] Copying project to /tmp/app...");
                }
                let copy_command = format!("cp -r {} /tmp/app", _path);
                execute_command(&copy_command, true).await;
            }
        }
        let mut _path = format!("/tmp/app");
        if let Some(commit_id) = _commit_id {
            let checkout_command = format!("cd {} && git checkout {}", _path, commit_id);
            execute_command(&checkout_command, true).await;

            let copy_command = format!("mkdir -p /tmp/new_code");
            execute_command(&copy_command, true).await;
            let copy_command = format!("cd {} && git diff-tree --no-commit-id --name-only -r {} | xargs -I {{}} git ls-tree --name-only {} {{}} | xargs git archive --format=tar {} | tar -x -C /tmp/new_code", _path, commit_id, commit_id, commit_id);
            execute_command(&copy_command, true).await;
            // now run secret scan on /tmp/new_code folder
            _path = format!("/tmp/new_code");
        }
        if verbose && !no_install {
            println!("[+] Installing project dependencies...");
            self.install_project_dependencies(&_path, ignore_dirs.clone(), new_detect_manifests, no_install.clone(), root_only, build_args.clone(), verbose).await;
        }else{
            println!("[+] Skipping installation of project dependencies...");
        }
        let mut manifests = Vec::new();

        if !root_only {
            println!("[+] Searching for manifest files, you can ignore this by passing --root-only flag...");
            manifests = find_files_recursively(&_path, new_manifests, ignore_dirs).await;
            println!("Manifests found: {:?}", manifests);
        }else{
            println!("[+] Searching for manifest files in root directory...");
            for manifest in new_manifests.iter() {
                let manifest_path = format!("{}/{}", _path, manifest);
                if std::path::Path::new(&manifest_path).exists() {
                    manifests.push(manifest_path);
                }
            }
        }

        if manifests.len() == 0 {
            println!("[*] No manifest files found!");
            return;
        }
        let mut mainfest_sca_result: HashMap<String, serde_json::Map<String, Value>> = HashMap::new();
        for manifest in manifests.iter() {
            println!("[+] Running SCA scan on {} manifest file...", manifest);
            let file_name = manifest.split("/").last().unwrap();
            let folder_path = manifest.replace(file_name, "");
            let sca_command = format!("cd {} && osv-scanner --format json -L {}", folder_path, file_name);
            let sca_output = execute_command(&sca_command, true).await;
            let json_output = serde_json::from_str::<serde_json::Value>(&sca_output).unwrap();
            let json_output = json_output.as_object().unwrap().get("results").unwrap().as_array().unwrap();
            if json_output.len() > 0 {
                let json_output = json_output[0].as_object().unwrap();
                mainfest_sca_result.insert(format!("{}/{}", folder_path, file_name), json_output.clone());
            } else {
                println!("[*] No vulnerabilities found in {} manifest file!", file_name);
                let manifest_path = format!("{}/{}", folder_path, file_name);
                let blank_vals = serde_json::from_str::<serde_json::Map<String, Value>>(format!("{{\"source\": {{\"path\": \"{}\", \"type\": \"lockfile\"}}, \"packages\": []}}", manifest_path).as_str()).unwrap();
                mainfest_sca_result.insert(format!("{}/{}", folder_path, file_name), blank_vals);
            }
        }
    
        if _server_url.is_some() {
            println!("[+] Posting SCA scan data to server...");
            let post_link = format!("{}/sca", _server_url.unwrap());
            let post_data = post_json_data(&post_link, json!(mainfest_sca_result.clone())).await;
            if verbose {
                if post_data.get("status").unwrap() == "200 OK" {
                    println!("[+] Successfully posted SCA scan data to server!");
                }else{
                    println!("Error while posting SCA scan data to server!");
                }
            }
        }
        // save data in output.json and before that get json data from output.json file if it exists and then append new data to it
        // output.json data will be in format {"sast":{}, "sca":{}, "secret":{}, "license":{}}
        let mut output_json = json!({});
        if std::path::Path::new("/tmp/output.json").exists() {
            let output_json_data = std::fs::read_to_string("/tmp/output.json").unwrap();
            output_json = serde_json::from_str::<serde_json::Value>(&output_json_data).unwrap();
        }
        output_json["sca"] = json!(mainfest_sca_result);
        std::fs::write("/tmp/output.json", serde_json::to_string_pretty(&output_json).unwrap()).unwrap();
    }
}
