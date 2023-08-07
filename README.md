## Code Security Tool
This tool helps in running Static Code Analysis (SCA), Static Application Security Testing (SAST), Secret scanning, and License compliance scanning on your project. It also allows you to write your own policy files in YAML format to enforce blocking in pipelines when security issues are detected.

### Docker Installation
To use the tool without building from source and installing Rust dependencies, you can run it using Docker. Follow the instructions below:

1. Pull the Docker image:

```shell
docker pull rohitcoder/code-security
```

2. Run the tool using Docker:

```shell
docker run rohitcoder/code-security <tool-options>
```

Add any Docker options you may need (e.g., volume mounting), and <tool-options> with the desired tool options explained in the next section.

### Usage
To run the Code Security Tool, use the following command:

```shell
docker run rohitcoder/code-security --path <path> --license-compliance --sast --sca --secret --license-compliance --policy-url <policy_url> --verbose
```
Replace ``<path>`` with the path to your project, which can be either a local folder path or a Git repository URL. If you want to use it with a private repository, provide the Git repository path with an access token.

Replace ``<policy_url> ``with the URL of your policy file in YAML format. This file defines rules for blocking pipelines when specific security issues are detected.

The tool will execute the specified scans (``--license-compliance``, ``--sast``, ``--sca``, ``--secret``) on your project and enforce the policies defined in the policy file. Verbose mode (``--verbose``) will provide detailed output.

Note: The API endpoints and start-server functionality are currently in development and not available.

## Building & Installation from Source

Clone and build the project:

```shell
git clone https://github.com/rohitcoder/code-security.git
cd code-security
cargo build --release
```

## CLI Usage
To use the tool from the command line, run the following command:

```shell
cargo run -- [options]
```
Replace ``[options]`` with the desired options from the list below.

### Options
<table>
   <thead>
      <tr>
         <th>Option</th>
         <th>Description</th>
      </tr>
   </thead>
   <tbody>
      <tr>
         <td>-v, --verbose</td>
         <td>Enable verbose mode.</td>
      </tr>
      <tr>
         <td>
            -p 
            <path>
            , --code-path 
            <path>
         </td>
         <td>Pass the path of the project to scan (local path or HTTP Git URL).</td>
      </tr>
      <tr>
         <td>
            -t 
            <path>
            , --rule-path
            <path>
         </td>
         <td>Pass the path of the semgrep rules repository (local path or HTTP Git URL).</td>
      </tr>
      <tr>
         <td>
            -n 
            <path>
            , --no-install
            <path>
         </td>Use this option to skip installation of project during SCA scan (Useful when you already have lock files in repo, and you want to save time).</td>
      </tr>
      <tr>
         <td>
            -r 
            <path>
            , --root-only
            <path>
         </td>Pass this flag, if you want to run SCA for only root folder manifests.</td>
      </tr>
      <tr>
         <td>
            -d
            <path>
            , --build-args
            <path>
         </td>Provide any additional build arguments for SCA scan (This will be injected in build commands like mvn build or npm run)</td>
      </tr>
      <tr>
         <td>
            -
            <path>
            , --manifests
            <path>
         </td>Pass list of manifests type to scan (comma separated values). Example: --manifests packages-lock.json,requirements.txt</td>
      </tr>
      <tr>
         <td>
            -i 
            <commit_id>
            , --commit-id 
            <commit_id>
         </td>
         <td>Pass the commit ID to scan (optional).</td>
      </tr>
      <tr>
         <td>
            -b 
            <branch>
            , --branch 
            <branch>
         </td>
         <td>Pass the branch name to scan (optional).</td>
      </tr>
      <tr>
         <td>-s, --sast</td>
         <td>Run SAST scan.</td>
      </tr>
      <tr>
         <td>
            -u 
            <server_url>
            , --server-url 
            <server_url>
         </td>
         <td>Pass the server URL to post scan results.</td>
      </tr>
      <tr>
         <td>-c, --sca</td>
         <td>Run SCA scan.</td>
      </tr>
      <tr>
         <td>-e, --secret</td>
         <td>Run Secret scan.</td>
      </tr>
      <tr>
         <td>-l, --license-compliance</td>
         <td>Run License Compliance scan.</td>
      </tr>
      <tr>
         <td>-j, --json</td>
         <td>Print JSON output. Note: This won't work with pipeline check implementation.</td>
      </tr>
      <tr>
         <td>
            -y 
            <policy_url>
            , --policy-url 
            <policy_url>
         </td>
         <td>Pass the policy URL to check if the pipeline should fail.</td>
      </tr>
      <tr>
         <td>-a, --start-server</td>
         <td>Start the API server.</td>
      </tr>
   </tbody>
</table>

## Example working command
```shell
docker run rohitcoder/code-security --path https://github.com/appsecco/dvja --license-compliance --sast --sca --secret --license-compliance --policy-url https://raw.githubusercontent.com/rohitcodergroww/cicd-policies/main/policy.yaml --verbose
```

## 💪 Contributors
Thank you for continuously making this tool better! 🙏

<a href="https://github.com/rohitcoder/code-security/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=rohitcoder/code-security" />
</a>

### Contribute

Please go through the [contributing guidelines](https://github.com/rohitcoder/code-security/blob/main/CONTRIBUTING.md) before you start, and let us know if you have any challenges or questions.


**Code Security** is maintained by [Rohit Kumar (@rohitcoder)](https://github.com/rohitcoder)

Thank you!
