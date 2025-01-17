use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::process::Command;
use std::fs;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use tokio;
use ignore::Walk;
use config::{Config, File};

#[derive(Debug, Deserialize)]
struct Settings {
    #[serde(default = "default_ollama_url")]
    ollama_url: String,
    #[serde(default = "default_model")]
    model: String,
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_model() -> String {
    "codellama".to_string()
}

#[derive(Debug)]
struct CodeReviewTool {
    ollama_url: String,
    model: String,
    client: Client,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    #[serde(default)]
    response: String,
    #[serde(default)]
    done: bool,
}

impl CodeReviewTool {
    fn new(ollama_url: Option<String>, model: Option<String>) -> Self {
        CodeReviewTool {
            ollama_url: ollama_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            model: model.unwrap_or_else(|| "codellama".to_string()),
            client: Client::new(),
        }
    }

    async fn get_git_diff(&self, path: &str, staged: bool) -> Result<String, Box<dyn Error>> {
        let mut cmd = Command::new("git");
        cmd.arg("diff");
        
        if staged {
            cmd.arg("--staged");
        }
        
        cmd.arg(path);
        
        let output = cmd.output()?;
        Ok(String::from_utf8(output.stdout)?)
    }

    fn tokenize_codebase(&self, root_path: &Path) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let mut codebase = HashMap::new();
        
        for entry in Walk::new(root_path) {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    
                    if path.is_file() {
                        match fs::read_to_string(path) {
                            Ok(content) => {
                                if let Ok(relative) = path.strip_prefix(root_path) {
                                    codebase.insert(relative.to_string_lossy().into_owned(), content);
                                }
                            },
                            Err(e) => {
                                eprintln!("Warning: Could not read file {}: {}", path.display(), e);
                                continue;
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Warning: Error accessing path: {}", e);
                    continue;
                }
            }
        }
        
        if codebase.is_empty() {
            eprintln!("Warning: No readable files found in the codebase");
        }
        
        Ok(codebase)
    }

    async fn review_changes(
        &self,
        diff: String,
        codebase_context: HashMap<String, String>,
        max_files_context: usize,
    ) -> Result<String, Box<dyn Error>> {
        let mut prompt = format!(
            "As a code reviewer, analyze the following changes:\n\n```diff\n{}\n```\n\n",
            diff
        );

        prompt.push_str("Relevant files from the codebase for context:\n\n");

        for (filename, content) in codebase_context.iter().take(max_files_context) {
            prompt.push_str(&format!("{}:\n```\n{}\n```\n\n", filename, content));
        }

        prompt.push_str("\nPlease provide a detailed code review focusing on:\n\
            1. Potential bugs or issues\n\
            2. Code style and best practices\n\
            3. Performance implications\n\
            4. Security considerations\n\
            5. Suggestions for improvement");

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
        };

        let response = self.client
            .post(format!("{}/api/generate", self.ollama_url))
            .json(&request)
            .send()
            .await?;
            
        // Get status before consuming response with text()
        let status = response.status();
        let text = response.text().await?;
        
        // Debug logging when DEBUG=TRUE
        if std::env::var("DEBUG").unwrap_or_default() == "TRUE" {
            eprintln!("Response status: {}", status);
            eprintln!("Raw response: {}", text);
        }

        // Parse line by line as each line is a separate JSON object
        let mut full_response = String::new();
        for line in text.lines() {
            if let Ok(resp) = serde_json::from_str::<OllamaResponse>(line) {
                full_response.push_str(&resp.response);
                if resp.done {
                    break;
                }
            }
        }

        Ok(full_response)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration
    let config = Config::builder()
        .add_source(File::with_name("config").required(false))
        .add_source(File::with_name("config.toml").required(false))
        .build()?;

    let settings: Settings = config.try_deserialize().unwrap_or_else(|_| Settings {
        ollama_url: default_ollama_url(),
        model: default_model(),
    });

    let reviewer = CodeReviewTool::new(
        Some(settings.ollama_url),
        Some(settings.model)
    );
    
    // Get codebase context
    let codebase = reviewer.tokenize_codebase(Path::new("./"))?;
    
    // Get current changes
    let diff = reviewer.get_git_diff(".", false).await?;
    
    // Get review
    let review = reviewer.review_changes(diff, codebase, 5).await?;
    println!("\nCode Review Results:");
    println!("{}", review);
    
    Ok(())
}