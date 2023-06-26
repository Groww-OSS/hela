use crate::utils::{common::execute_command, file_utils::find_files_recursively};

pub struct ScaTool;

pub static SUPPORTED_MANIFESTS : [&str; 2] = [
    "requirements.txt",
    "package-lock.json",
];

pub static DETECT_MANIFESTS : [&str; 2] = [
    "requirements.txt",
    "package.json",
];

impl ScaTool {
    pub fn new() -> Self {
        ScaTool
    }

    async fn install_project_dependencies(&self, _path: &str, ignore_dirs: Vec<&str>) {
        // detect if project is python or nodejs based on manifest from DETECT_MANIFESTS and install that
        // installation script for each language would be in /tmp/install/ folder with file like python.sh, javascript.sh, java.sh etc for each language, developer need to write that script and we will execute it here based on language detection
        let installation_script_path = format!("{}/install", std::env::temp_dir().to_str().unwrap());
        
        let detected_files = find_files_recursively(_path, DETECT_MANIFESTS.to_vec(), ignore_dirs).await;
        let language_mapping = {
            let mut map = std::collections::HashMap::new();
            map.insert("requirements.txt", "python");
            map.insert("package.json", "javascript");
            map.insert("package-lock.json", "javascript");
            map.insert("pom.xml", "maven");
            map.insert("build.gradle", "gradle");
            map.insert("go.mod", "go");
            map
        };
        // check if we have one of manifest file from DETECT_MANIFESTS then install dependencies based on language_mapping
        if detected_files.len() > 0 {
           for detected_file in detected_files.iter() {
             let file_name = detected_file.split("/").last().unwrap();
             let folder_path = detected_file.replace(file_name, "");
             let language = language_mapping.get(file_name).unwrap().to_string();
             if language == "python" {
                 println!("Installing python dependencies...");
                 let install_command = format!("cd {} && pip install -r {}", folder_path, file_name);
                 execute_command(&install_command, true).await;
                 // check if installation script exists for python and then execute it
                 if std::path::Path::new(&format!("{}/python.sh", installation_script_path)).exists() {
                    println!("[INFO] Found python installation script, executing it...");
                    let install_command = format!("cd {} && sh {}/python.sh", folder_path, installation_script_path);
                    execute_command(&install_command, true).await;
                 }
             }
             if language == "javascript" {
                // check if installation script exists for python and then execute it
                 if std::path::Path::new(&format!("{}/javascript.sh", installation_script_path)).exists() {
                    println!("[INFO] Found javascript installation script, executing it...");
                    let install_command = format!("cd {} && sh {}/javascript.sh", folder_path, installation_script_path);
                    execute_command(&install_command, true).await;
                 }
                 println!("Installing javascript dependencies...");
                 let install_command = format!("cd {} && npm install --force --ignore-scripts", folder_path);
                 execute_command(&install_command, true).await;
             }

             if language == "maven" {
                // check if installation script exists for python and then execute it
                 if std::path::Path::new(&format!("{}/java.sh", installation_script_path)).exists() {
                    println!("[INFO] Found java installation script, executing it...");
                    let install_command = format!("cd {} && sh {}/java.sh", folder_path, installation_script_path);
                    execute_command(&install_command, true).await;
                 }
                 println!("Installing maven dependencies...");
                 let install_command = format!("cd {} && mvn install", folder_path);
                 execute_command(&install_command, true).await;
             }

             if language == "gradle" {
                // check if installation script exists for python and then execute it
                 if std::path::Path::new(&format!("{}/java.sh", installation_script_path)).exists() {
                    println!("[INFO] Found java installation script, executing it...");
                    let install_command = format!("cd {} && sh {}/java.sh", folder_path, installation_script_path);
                    execute_command(&install_command, true).await;
                 }
                 println!("Installing gradle dependencies...");
                 let install_command = format!("cd {} && gradle build", folder_path);
                 execute_command(&install_command, true).await;
             }
           }
        }
    }

    pub async fn run_scan(&self, _path: &str, _commit_id: Option<&str>) {
        println!("Running SCA scan on path: {}", _path);
        
        let mut ignore_dirs = Vec::new();
        ignore_dirs.push("node_modules");
        ignore_dirs.push("bin");
        ignore_dirs.push("venv");
        ignore_dirs.push(".venv");

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
        println!("Installing project dependencies...");
        self.install_project_dependencies(&_path, ignore_dirs.clone()).await;
        println!("Running SCA scan on path: {}", _path);
        let manifests = find_files_recursively(&_path, SUPPORTED_MANIFESTS.to_vec(), ignore_dirs).await;
        println!("Found manifests: {:?}", manifests);
        for manifest in manifests.iter() {
            let file_name = manifest.split("/").last().unwrap();
            let folder_path = manifest.replace(file_name, "");
            let sca_command = format!("cd {} && osv-scanner --format json -L {}", folder_path, file_name);
            let sca_output = execute_command(&sca_command, true).await;
            println!("Sca output: {}", sca_output);
        }
    }
}
