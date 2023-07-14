mod scans;
mod utils;
mod api;

use std::{env, process::exit, collections::HashMap, hash::Hash};
use prettytable::{Table, row};
use scans::scanner::ScanRunner;
use crate::scans::tools::{sast_tool::SastTool, sca_tool::ScaTool, secret_tool::SecretTool, license_tool::LicenseTool};
use actix_web::{App, HttpServer};
use dotenv::dotenv;
use argparse::{ArgumentParser, StoreTrue, Store};

async fn execute_scan(scan_type: &str, path: &str, commit_id: Option<&str>, branch: Option<&str>, server_url: Option<&str>, verbose: bool) {
    let scanner = ScanRunner::new(
        SastTool::new(),
        ScaTool::new(),
        SecretTool::new(),
        LicenseTool::new(),
    );

    scanner.execute_scan(scan_type, path, commit_id, branch, server_url, verbose).await;
}

async fn start_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .configure(api::scan::config)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

#[actix_web::main]
async fn main() {
    dotenv().ok();
    // Parse command-line arguments
    let mut is_sast = false;
    let mut is_sca = false;
    let mut is_secret = false;
    let mut is_license_compliance = false;
    let mut is_start_server = false;
    let mut verbose = false;
    let mut path = String::new();
    let mut commit_id = String::new();
    let mut server_url = String::new();
    let mut branch = String::new();
    let mut policy_url = String::new();
    let mut json = false;

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Scan CLI tool");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue, "Enable verbose mode!");
        ap.refer(&mut path)
            .add_option(&["-p", "--path"], Store, "Pass the path of the project to scan (Local Path or HTTP Git URL)");
        ap.refer(&mut commit_id)
            .add_option(&["-i", "--commit-id"], Store, "Pass the commit ID to scan (Optional)");
        ap.refer(&mut branch)
            .add_option(&["-b", "--branch"], Store, "Pass the branch name to scan (Optional)");
        ap.refer(&mut is_sast)
            .add_option(&["-s", "--sast"], StoreTrue, "Run SAST scan");
        ap.refer(&mut server_url)
            .add_option(&["-u", "--server-url"], Store, "Pass the server URL to post scan results");
        ap.refer(&mut is_sca)
            .add_option(&["-c", "--sca"], StoreTrue, "Run SCA scan");
        ap.refer(&mut is_secret)
            .add_option(&["-e", "--secret"], StoreTrue, "Run Secret scan");
        ap.refer(&mut is_license_compliance)
            .add_option(&["-l", "--license-compliance"], StoreTrue, "Run License Compliance scan");
        ap.refer(&mut json)
            .add_option(&["-j", "--json"], StoreTrue, "Print JSON output");
        ap.refer(&mut policy_url)
            .add_option(&["-y", "--policy-url"], Store, "Pass the policy url to check if pipeline should fail");
        ap.refer(&mut is_start_server)
            .add_option(&["-a", "--start-server"], StoreTrue, "Start API server");
        ap.parse_args_or_exit();
    }

    if verbose {
        println!("[+] Verbose mode enabled!");
    }

    if is_start_server {
        println!("Starting API server...");
        if let Err(err) = start_server().await {
            println!("Failed to start API server: {}", err);
            exit(1)
        }
        println!("API server started successfully!");
    }

    if is_sast {
        execute_scan("sast", &path, if commit_id.is_empty() { None } else { Some(&commit_id) },  if branch.is_empty() { None } else { Some(&branch) }, if server_url.is_empty() { None } else { Some(&server_url) }, verbose).await;
    }

    if is_sca {
        execute_scan("sca", &path, if commit_id.is_empty() { None } else { Some(&commit_id) }, if branch.is_empty() { None } else { Some(&branch) }, if server_url.is_empty() { None } else { Some(&server_url) }, verbose).await;
    }

    if is_secret {
        execute_scan("secret", &path, if commit_id.is_empty() { None } else { Some(&commit_id) }, if branch.is_empty() { None } else { Some(&branch) }, if server_url.is_empty() { None } else { Some(&server_url) }, verbose).await;
    }

    if is_license_compliance {
        execute_scan("license-compliance", &path, if commit_id.is_empty() { None } else { Some(&commit_id) }, if branch.is_empty() { None } else { Some(&branch) }, if server_url.is_empty() { None } else { Some(&server_url) }, verbose).await;
    }

    if !is_start_server && !is_sast && !is_sca && !is_secret && !is_license_compliance {
        println!("Invalid command. Available commands: start-server, sast, sca, secret, license-compliance");
    }

    if json {
        if std::path::Path::new("/tmp/output.json").exists() {
            let output = std::fs::read_to_string("/tmp/output.json").unwrap();
            println!("{}", output);
        }
    }

    let mut pipeline_sast_sca_data = HashMap::new();
    let mut pipeline_secret_license_data = HashMap::new();
    // pipeline failure specific data format
    /*
        {
            "sast": {
                "high_count": 0,
                "critical_count": 0,
                "medium_count": 0,
                "low_count": 0,
                "info_count": 0,
            },
            "sca": {
                "high_count": 0,
                "critical_count": 0,
                "medium_count": 0,
                "low_count": 0,
                "info_count": 0,
            },
            "secret": {
                "JDBC": 1,
                "AWS": 2,
            },
            "license": {
                "licenses": [
                    "AGPL",
                    "GPL",
                    "LGPL",
                    "Apache-2.0"
                ]
            },
        }
     */
    // now lets clean the result and store only required data
    let original_output = std::fs::read_to_string("/tmp/output.json").unwrap();
    let json_output: serde_json::Value = serde_json::from_str(&original_output).expect("Error parsing JSON");
    
    // start preparing results here
    let mut sast_results = Vec::new();
    if is_sast {

      let mut pipeline_sast_data: HashMap<&str, i64> = HashMap::new();
      let mut warning_count = 0;
      let mut info_count = 0;
      let mut error_count = 0;
      let mut critical_count = 0;
      let mut high_count = 0;
      let mut medium_count = 0;
      let mut low_count = 0;

      for result in json_output["sast"].as_array().unwrap() {
          let mut sast_result = HashMap::new();
          sast_result.insert("check_id", result["check_id"].as_str().unwrap());
          sast_result.insert("path", result["path"].as_str().unwrap());
          sast_result.insert("severity", result["extra"]["severity"].as_str().unwrap());
          sast_result.insert("message", result["extra"]["message"].as_str().unwrap());
          sast_result.insert("lines", result["extra"]["lines"].as_str().unwrap());
          
          if result["extra"]["severity"].as_str().unwrap().to_lowercase() == "warning" {
              warning_count += 1;
          } else if result["extra"]["severity"].as_str().unwrap().to_lowercase() == "info" {
              info_count += 1;
          } else if result["extra"]["severity"].as_str().unwrap().to_lowercase() == "error" {
              error_count += 1;
          } else if result["extra"]["severity"].as_str().unwrap().to_lowercase() == "critical" {
              critical_count += 1;
          } else if result["extra"]["severity"].as_str().unwrap().to_lowercase() == "high" {
              high_count += 1;
          } else if result["extra"]["severity"].as_str().unwrap().to_lowercase() == "medium" {
              medium_count += 1;
          } else if result["extra"]["severity"].as_str().unwrap().to_lowercase() == "low" {
              low_count += 1;
          }
          sast_results.push(sast_result);
      }
      pipeline_sast_data.insert("high_count", high_count);
      pipeline_sast_data.insert("critical_count", critical_count);
      pipeline_sast_data.insert("medium_count", medium_count);
      pipeline_sast_data.insert("low_count", low_count);
      pipeline_sast_data.insert("info_count", info_count);
      pipeline_sast_data.insert("warning_count", warning_count);
      pipeline_sast_data.insert("error_count", error_count);


      let mut table = Table::new();
      println!("\n\n");
      println!("\t\t ================== SAST Results ==================");
      table.add_row(row![bFg->"S.No", bFg->"Path", bFg->"Severity", bFg->"Message"]);
      let mut sast_count = 0;
      for result in sast_results {
          sast_count += 1;
          // strip message to 50 characters
          table.add_row(row![sast_count, result["path"], result["severity"], result["message"].chars().take(50).collect::<String>()]);
      }
      table.printstd();

      pipeline_sast_sca_data.insert("sast", pipeline_sast_data);
    }

    if is_sca {
      
      let mut pipline_sca_data = HashMap::new();
      let mut warning_count = 0;
      let mut info_count = 0;
      let mut error_count = 0;
      let mut critical_count = 0;
      let mut high_count = 0;
      let mut medium_count = 0;
      let mut low_count = 0;

      // lets prepare list of Vulnerabilities with package name, version, ecosytem in each vulnerability
      for (manifest_file, sca_result) in json_output["sca"].as_object().unwrap() {
        let mut vulnerabilities = Vec::new();
        for package in sca_result["packages"].as_array().unwrap() {
            let mut vulnerability = HashMap::new();
            vulnerability.insert("package", package["package"]["name"].as_str().unwrap());
            vulnerability.insert("version", package["package"]["version"].as_str().unwrap());
            vulnerability.insert("ecosystem", package["package"]["ecosystem"].as_str().unwrap());

            for vuln in package["vulnerabilities"].as_array().unwrap() {
                let mut severity = vuln["database_specific"]["severity"].as_str().unwrap();
                if severity == "MODERATE" {
                    severity = "MEDIUM";
                }
                vulnerability.insert("summary", vuln["summary"].as_str().unwrap());
                vulnerability.insert("details", vuln["details"].as_str().unwrap());
                vulnerability.insert("severity", severity.clone());

                let cwe_id_array = vuln["database_specific"]["cwe_ids"].as_array().unwrap();
                if cwe_id_array.len() > 0 {
                    vulnerability.insert("cwe_id", cwe_id_array[0].as_str().unwrap());
                }else{
                    vulnerability.insert("cwe_id", "");
                }
                
                let aliases_array = vuln["aliases"].as_array().unwrap();
                if aliases_array.len() > 0 {
                    vulnerability.insert("aliases", aliases_array[0].as_str().unwrap());
                }else{
                    vulnerability.insert("aliases", "");
                }

                if severity.to_lowercase() == "warning" {
                    warning_count += 1;
                } else if severity.to_lowercase() == "info" {
                    info_count += 1;
                } else if severity.to_lowercase() == "error" {
                    error_count += 1;
                } else if severity.to_lowercase() == "critical" {
                    critical_count += 1;
                } else if severity.to_lowercase() == "high" {
                    high_count += 1;
                } else if severity.to_lowercase() == "medium" {
                    medium_count += 1;
                } else if severity.to_lowercase() == "low" {
                    low_count += 1;
                }
            }

            vulnerabilities.push(vulnerability);
        }

            println!("\n\n");
            println!("\t\t ================== SCA Results for {} ==================", manifest_file);

            let mut table = Table::new();
            table.add_row(row![bFg->"S.No", bFg->"Package", bFg->"Severity", bFg->"Summary", bFg->"CWE ID", bFg->"Aliases"]);
            let mut sca_count = 0;

            for result in vulnerabilities {
                sca_count += 1;
                // strip summary to 50 characters
                table.add_row(row![sca_count, format!("{}@{}", result["package"], result["version"]), result["severity"], result["summary"].chars().take(50).collect::<String>(), result["cwe_id"], result["aliases"]]);
            }
            table.printstd();
        }

        pipline_sca_data.insert("high_count", high_count);
        pipline_sca_data.insert("critical_count", critical_count);
        pipline_sca_data.insert("medium_count", medium_count);
        pipline_sca_data.insert("low_count", low_count);
        pipline_sca_data.insert("info_count", info_count);
        pipline_sca_data.insert("warning_count", warning_count);
        pipline_sca_data.insert("error_count", error_count);

        pipeline_sast_sca_data.insert("sca", pipline_sca_data);
    }

    if is_secret {

      let mut detected_detectors = Vec::new();

      let mut secret_results = HashMap::new();
      for result in json_output["secret"]["results"].as_array().unwrap() {
          let mut secret_result = HashMap::new();
          secret_result.insert("file", result["SourceMetadata"]["Data"]["Filesystem"]["file"].as_str().unwrap());
          if result["SourceMetadata"]["Data"]["Filesystem"]["line"].is_string() {
            secret_result.insert("line", result["SourceMetadata"]["Data"]["Filesystem"]["line"].as_str().unwrap());
           }else{
                secret_result.insert("line", "");
           }
          secret_result.insert("raw", result["Raw"].as_str().unwrap());
          secret_result.insert("detector_name", result["DetectorName"].as_str().unwrap());
          secret_result.insert("decoder_name", result["DecoderName"].as_str().unwrap());
          secret_results.insert("results", secret_result);

          if !detected_detectors.contains(&result["DetectorName"].as_str().unwrap().to_string()) {
            detected_detectors.push(result["DetectorName"].as_str().unwrap().to_string());
          }
          
      }

      pipeline_secret_license_data.insert("detected_detectors", detected_detectors);


      println!("\n\n");
      println!("\t\t ================== Secret Results ==================");
  
      let mut table = Table::new();
      table.add_row(row![bFg->"S.No", bFg->"File", bFg->"Line", bFg->"Raw", bFg->"Detector Name"]);
      let mut secret_count = 0;
  
      for (_key, value) in secret_results {
          secret_count += 1;
          table.add_row(row![secret_count, value["file"], value["line"], value["raw"], value["detector_name"]]);
      }
      table.printstd();
    }

    if is_license_compliance {

        let mut licenses_list = Vec::new();

        let mut license_results = HashMap::new();
        for (manifest, license_detail) in json_output["license"].as_object().unwrap() {
          
          let mut detected_licenses = Vec::new();
          for (package_name, licenses) in license_detail.as_object().unwrap() {
            let mut license_details = HashMap::new();
            license_details.insert(package_name, licenses.clone());
            detected_licenses.push(license_details.get(package_name).unwrap().clone());
          }
          license_results.insert(manifest, license_detail);

          for license in detected_licenses.iter() {
            for license in license.as_array().unwrap() {
                if !licenses_list.contains(&license.as_str().unwrap().to_string()) {
                    licenses_list.push(license.as_str().unwrap().to_string());
                }
            }
          }
          
          println!("\n\n");
          println!("\t\t ================== License Details for {} ==================", manifest);

          let mut table = Table::new();
          table.add_row(row![bFg->"S.No", bFg->"Package", bFg->"Licenses"]);
          let mut license_count = 0;
          for (package_name, licenses) in license_detail.as_object().unwrap() {
            license_count += 1;
            // lets create license arary
            let mut license_array = Vec::new();
            for license in licenses.as_array().unwrap() {
                license_array.push(license.as_str().unwrap());
            }
            table.add_row(row![license_count, package_name, license_array.join(", ")]);
          }
          table.printstd();
      }
      licenses_list = licenses_list.iter().map(|x| x.to_lowercase()).collect::<Vec<String>>();
      pipeline_secret_license_data.insert("licenses", licenses_list);
    }

    // Policy implementation
    if !policy_url.is_empty() {
        let mut is_pipeline_failed = false;
        let mut pipeline_failure_reason = String::new();
        let policy_yaml = reqwest::get(policy_url).await.unwrap().text().await.unwrap();
        let policy_yaml: serde_yaml::Value = serde_yaml::from_str(&policy_yaml).unwrap();
        let policy_json = policy_yaml.as_mapping().unwrap();
        let mut sast_policy = None;
        let mut sca_policy = None;
        let mut secret_policy = None;
        let mut license_policy = None;

        for (key, value) in policy_json {
            if key.as_str().unwrap() == "sast" {
                sast_policy = Some(value);
            }
            if key.as_str().unwrap() == "sca" {
                sca_policy = Some(value);
            }
            if key.as_str().unwrap() == "secret" {
                secret_policy = Some(value);
            }
            if key.as_str().unwrap() == "license" {
                license_policy = Some(value);
            }
        }

        // now lets write logic to check policy against scan results since we have all data in pipeline_sast_sca_data and pipeline_secret_license_data

        if is_sast && sast_policy.is_some() {
            let sast_policy = sast_policy.unwrap().as_mapping().unwrap();
            for (key, value) in sast_policy {
                let key = key.as_str().unwrap();
                let value = value.as_mapping().unwrap();
                let operator = value.get(&serde_yaml::Value::String("operator".to_string())).unwrap().as_str().unwrap();
                let value = value.get(&serde_yaml::Value::String("value".to_string())).unwrap().as_i64().unwrap();
                let pipeline_sast_data = pipeline_sast_sca_data.get("sast").unwrap();
                let pipeline_sast_data = pipeline_sast_data.get(key).unwrap();
                if operator == "greater_than" {
                    if pipeline_sast_data > &value {
                        is_pipeline_failed = true;
                        pipeline_failure_reason = format!("Pipeline failed because {} count is {} which is greater than {}", key, pipeline_sast_data, value);
                    }
                }else if operator == "less_than" {
                    if pipeline_sast_data < &value {
                        is_pipeline_failed = true;
                        pipeline_failure_reason = format!("Pipeline failed because {} count is {} which is less than {}", key, pipeline_sast_data, value);
                    }
                }else if operator == "equal_to" {
                    if pipeline_sast_data == &value {
                        is_pipeline_failed = true;
                        pipeline_failure_reason = format!("Pipeline failed because {} count is {} which is equal to {}", key, pipeline_sast_data, value);
                    }
                }
            }
        }

        if is_sca && sca_policy.is_some() {
            let sca_policy = sca_policy.unwrap().as_mapping().unwrap();
            for (key, value) in sca_policy {
                let key = key.as_str().unwrap();
                let value = value.as_mapping().unwrap();
                let operator = value.get(&serde_yaml::Value::String("operator".to_string())).unwrap().as_str().unwrap();
                let value = value.get(&serde_yaml::Value::String("value".to_string())).unwrap().as_i64().unwrap();
                let pipeline_sca_data = pipeline_sast_sca_data.get("sca").unwrap();
                let pipeline_sca_data = pipeline_sca_data.get(key).unwrap();
                if operator == "greater_than" {
                    if pipeline_sca_data > &value {
                        is_pipeline_failed = true;
                        pipeline_failure_reason = format!("Pipeline failed because {} count is {} which is greater than {}", key, pipeline_sca_data, value);
                    }
                }else if operator == "less_than" {
                    if pipeline_sca_data < &value {
                        is_pipeline_failed = true;
                        pipeline_failure_reason = format!("Pipeline failed because {} count is {} which is less than {}", key, pipeline_sca_data, value);
                    }
                }else if operator == "equal_to" {
                    if pipeline_sca_data == &value {
                        is_pipeline_failed = true;
                        pipeline_failure_reason = format!("Pipeline failed because {} count is {} which is equal to {}", key, pipeline_sca_data, value);
                    }
                }
            }
        }

        if is_secret && secret_policy.is_some() {
            let secret_policy = secret_policy.unwrap().as_mapping().unwrap();
            if secret_policy.contains_key(&serde_yaml::Value::String("contains".to_string())) {
                let contains = secret_policy.get(&serde_yaml::Value::String("contains".to_string())).unwrap().as_sequence().unwrap();
                let pipeline_secret_data = pipeline_secret_license_data.get("detected_detectors").unwrap();
                for detector in pipeline_secret_data.iter() {
                    if contains.contains(&serde_yaml::Value::String(detector.to_string())) {
                        is_pipeline_failed = true;
                        pipeline_failure_reason = format!("Pipeline failed because {} is present in blocked list", detector);
                    }
                }
            }
        }

        if is_license_compliance && license_policy.is_some() {
            let license_policy = license_policy.unwrap().as_mapping().unwrap();
            if license_policy.contains_key(&serde_yaml::Value::String("contains".to_string())) {
                let contains = license_policy.get(&serde_yaml::Value::String("contains".to_string())).unwrap().as_sequence().unwrap();
                let contains = contains.iter().map(|x| x.as_str().unwrap().to_lowercase()).collect::<Vec<String>>();
                let pipeline_license_data = pipeline_secret_license_data.get("licenses").unwrap();
                for license in pipeline_license_data.iter() {
                    if contains.contains(&license.to_string().to_lowercase()) { 
                        is_pipeline_failed = true;
                        pipeline_failure_reason = format!("Pipeline failed because {} license is present in blocked list", license);
                    }
                }
            }
        }

        if is_pipeline_failed {
            println!("\n\n");
            println!("\t\t ================== ❌ Pipeline Failed ==================");
            println!("\t\t Reason: {}", pipeline_failure_reason);
            println!("\n\n");
            // finish everything and smoothly exit
            exit(1);
        }else{
            println!("\n\n");
            println!("\t\t ================== ✅ Pipeline Passed ==================");
            println!("\n\n");
        }
    }
}
