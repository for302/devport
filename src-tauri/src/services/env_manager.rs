use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EnvProfileType {
    Development,
    Staging,
    Production,
    Custom(String),
}

impl EnvProfileType {
    pub fn to_file_suffix(&self) -> String {
        match self {
            EnvProfileType::Development => "development".to_string(),
            EnvProfileType::Staging => "staging".to_string(),
            EnvProfileType::Production => "production".to_string(),
            EnvProfileType::Custom(name) => name.to_lowercase().replace(' ', "-"),
        }
    }

    pub fn from_file_name(file_name: &str) -> Self {
        match file_name {
            ".env" => EnvProfileType::Development,
            ".env.development" => EnvProfileType::Development,
            ".env.staging" => EnvProfileType::Staging,
            ".env.production" => EnvProfileType::Production,
            name if name.starts_with(".env.") => {
                let suffix = name.strip_prefix(".env.").unwrap_or("");
                match suffix {
                    "development" | "dev" => EnvProfileType::Development,
                    "staging" | "stage" => EnvProfileType::Staging,
                    "production" | "prod" => EnvProfileType::Production,
                    _ => EnvProfileType::Custom(suffix.to_string()),
                }
            }
            _ => EnvProfileType::Custom(file_name.to_string()),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            EnvProfileType::Development => "Development".to_string(),
            EnvProfileType::Staging => "Staging".to_string(),
            EnvProfileType::Production => "Production".to_string(),
            EnvProfileType::Custom(name) => name.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVariable {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvProfile {
    pub name: String,
    pub file_name: String,
    pub variables: Vec<EnvVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    pub name: String,
    pub file_name: String,
    pub profile_type: EnvProfileType,
    pub is_active: bool,
    pub variable_count: usize,
    pub last_modified: Option<u64>,
}

const ACTIVE_PROFILE_FILE: &str = ".env.active";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDiff {
    pub key: String,
    pub value_a: String,
    pub value_b: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileComparison {
    pub profile_a: String,
    pub profile_b: String,
    pub only_in_a: Vec<String>,
    pub only_in_b: Vec<String>,
    pub different_values: Vec<ProfileDiff>,
}

pub struct EnvManager {
    pub project_path: PathBuf,
}

impl EnvManager {
    pub fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }

    pub fn get_env_files(&self) -> Vec<String> {
        let mut env_files = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.project_path) {
            for entry in entries.filter_map(Result::ok) {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with(".env") {
                    env_files.push(file_name);
                }
            }
        }

        env_files.sort();
        env_files
    }

    pub fn read_env_file(&self, file_name: &str) -> Result<Vec<EnvVariable>, String> {
        let path = self.project_path.join(file_name);

        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut variables = Vec::new();
        let mut current_comment: Option<String> = None;

        for line in reader.lines().filter_map(Result::ok) {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                current_comment = None;
                continue;
            }

            if trimmed.starts_with('#') {
                current_comment = Some(trimmed[1..].trim().to_string());
                continue;
            }

            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_string();
                let value = trimmed[eq_pos + 1..].trim().to_string();

                let value = if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    value[1..value.len() - 1].to_string()
                } else {
                    value
                };

                let is_secret = Self::is_secret_key(&key);

                variables.push(EnvVariable {
                    key,
                    value,
                    is_secret,
                    comment: current_comment.take(),
                });
            }
        }

        Ok(variables)
    }

    pub fn write_env_file(
        &self,
        file_name: &str,
        variables: &[EnvVariable],
    ) -> Result<(), String> {
        let path = self.project_path.join(file_name);
        let mut content = String::new();

        for var in variables {
            if let Some(comment) = &var.comment {
                content.push_str(&format!("# {}\n", comment));
            }

            let value = if var.value.contains(' ')
                || var.value.contains('#')
                || var.value.contains('=')
            {
                format!("\"{}\"", var.value)
            } else {
                var.value.clone()
            };

            content.push_str(&format!("{}={}\n", var.key, value));
        }

        fs::write(&path, content).map_err(|e| e.to_string())
    }

    pub fn create_env_file(&self, file_name: &str) -> Result<(), String> {
        let path = self.project_path.join(file_name);

        if path.exists() {
            return Err(format!("File {} already exists", file_name));
        }

        File::create(&path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_env_file(&self, file_name: &str) -> Result<(), String> {
        let path = self.project_path.join(file_name);

        if !path.exists() {
            return Err(format!("File {} does not exist", file_name));
        }

        if file_name == ".env" {
            return Err("Cannot delete main .env file".to_string());
        }

        fs::remove_file(&path).map_err(|e| e.to_string())
    }

    pub fn copy_env_file(&self, source: &str, destination: &str) -> Result<(), String> {
        let source_path = self.project_path.join(source);
        let dest_path = self.project_path.join(destination);

        if !source_path.exists() {
            return Err(format!("Source file {} does not exist", source));
        }

        if dest_path.exists() {
            return Err(format!("Destination file {} already exists", destination));
        }

        fs::copy(&source_path, &dest_path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_profiles(&self) -> Vec<EnvProfile> {
        let env_files = self.get_env_files();
        let mut profiles = Vec::new();

        for file_name in env_files {
            let name = if file_name == ".env" {
                "Default".to_string()
            } else if file_name.starts_with(".env.") {
                file_name[5..].to_string()
            } else {
                file_name.clone()
            };

            let variables = self.read_env_file(&file_name).unwrap_or_default();

            profiles.push(EnvProfile {
                name,
                file_name,
                variables,
            });
        }

        profiles
    }

    fn is_secret_key(key: &str) -> bool {
        let secret_patterns = [
            "SECRET",
            "PASSWORD",
            "PASSWD",
            "PWD",
            "TOKEN",
            "API_KEY",
            "APIKEY",
            "AUTH",
            "CREDENTIAL",
            "PRIVATE",
            "KEY",
            "ENCRYPTION",
        ];

        let key_upper = key.to_uppercase();
        secret_patterns
            .iter()
            .any(|pattern| key_upper.contains(pattern))
    }

    /// List all available profiles for a project
    pub fn list_profiles(&self) -> Result<Vec<ProfileInfo>, String> {
        let env_files = self.get_env_files();
        let active_profile = self.get_active_profile().unwrap_or_else(|_| ".env".to_string());
        let mut profiles = Vec::new();

        for file_name in env_files {
            let profile_type = EnvProfileType::from_file_name(&file_name);
            let name = match &profile_type {
                EnvProfileType::Development if file_name == ".env" => "Default".to_string(),
                _ => profile_type.display_name(),
            };

            let path = self.project_path.join(&file_name);
            let last_modified = path
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());

            let variables = self.read_env_file(&file_name).unwrap_or_default();

            profiles.push(ProfileInfo {
                name,
                file_name: file_name.clone(),
                profile_type,
                is_active: file_name == active_profile,
                variable_count: variables.len(),
                last_modified,
            });
        }

        Ok(profiles)
    }

    /// Get the currently active profile file name
    pub fn get_active_profile(&self) -> Result<String, String> {
        let active_file = self.project_path.join(ACTIVE_PROFILE_FILE);

        if active_file.exists() {
            let content = fs::read_to_string(&active_file)
                .map_err(|e| format!("Failed to read active profile: {}", e))?;
            let profile = content.trim().to_string();

            // Verify the profile file exists
            let profile_path = self.project_path.join(&profile);
            if profile_path.exists() {
                return Ok(profile);
            }
        }

        // Default to .env if no active profile is set or if it doesn't exist
        Ok(".env".to_string())
    }

    /// Switch to a different profile (sets it as active)
    pub fn switch_profile(&self, profile_file_name: &str) -> Result<(), String> {
        let profile_path = self.project_path.join(profile_file_name);

        if !profile_path.exists() {
            return Err(format!("Profile {} does not exist", profile_file_name));
        }

        let active_file = self.project_path.join(ACTIVE_PROFILE_FILE);
        fs::write(&active_file, profile_file_name)
            .map_err(|e| format!("Failed to set active profile: {}", e))?;

        Ok(())
    }

    /// Create a new profile, optionally copying from an existing profile
    pub fn create_profile(
        &self,
        profile_type: EnvProfileType,
        copy_from: Option<&str>,
    ) -> Result<String, String> {
        let file_name = match &profile_type {
            EnvProfileType::Development => ".env.development".to_string(),
            EnvProfileType::Staging => ".env.staging".to_string(),
            EnvProfileType::Production => ".env.production".to_string(),
            EnvProfileType::Custom(name) => format!(".env.{}", name.to_lowercase().replace(' ', "-")),
        };

        let path = self.project_path.join(&file_name);

        if path.exists() {
            return Err(format!("Profile {} already exists", file_name));
        }

        if let Some(source) = copy_from {
            let source_path = self.project_path.join(source);
            if !source_path.exists() {
                return Err(format!("Source profile {} does not exist", source));
            }
            fs::copy(&source_path, &path)
                .map_err(|e| format!("Failed to copy profile: {}", e))?;
        } else {
            // Create empty file with a comment
            let content = format!("# {} environment variables\n", profile_type.display_name());
            fs::write(&path, content)
                .map_err(|e| format!("Failed to create profile: {}", e))?;
        }

        Ok(file_name)
    }

    /// Delete a profile
    pub fn delete_profile(&self, profile_file_name: &str) -> Result<(), String> {
        // Cannot delete .env (main file)
        if profile_file_name == ".env" {
            return Err("Cannot delete the main .env file".to_string());
        }

        let path = self.project_path.join(profile_file_name);

        if !path.exists() {
            return Err(format!("Profile {} does not exist", profile_file_name));
        }

        fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete profile: {}", e))?;

        // If this was the active profile, switch to .env
        if let Ok(active) = self.get_active_profile() {
            if active == profile_file_name {
                let _ = self.switch_profile(".env");
            }
        }

        Ok(())
    }

    /// Export a profile to a specified path as a .env file
    pub fn export_profile(&self, profile_file_name: &str, export_path: &str) -> Result<(), String> {
        let source_path = self.project_path.join(profile_file_name);

        if !source_path.exists() {
            return Err(format!("Profile {} does not exist", profile_file_name));
        }

        let dest_path = PathBuf::from(export_path);

        // Read the variables and write them to the export path
        let variables = self.read_env_file(profile_file_name)?;

        let mut content = String::new();
        content.push_str(&format!("# Exported from {}\n", profile_file_name));
        content.push_str(&format!("# Project: {}\n\n", self.project_path.display()));

        for var in &variables {
            if let Some(comment) = &var.comment {
                content.push_str(&format!("# {}\n", comment));
            }

            let value = if var.value.contains(' ')
                || var.value.contains('#')
                || var.value.contains('=')
            {
                format!("\"{}\"", var.value)
            } else {
                var.value.clone()
            };

            content.push_str(&format!("{}={}\n", var.key, value));
        }

        fs::write(&dest_path, content)
            .map_err(|e| format!("Failed to export profile: {}", e))?;

        Ok(())
    }

    /// Import a profile from an external file
    pub fn import_profile(
        &self,
        profile_type: EnvProfileType,
        import_path: &str,
    ) -> Result<String, String> {
        let source_path = PathBuf::from(import_path);

        if !source_path.exists() {
            return Err(format!("Import file {} does not exist", import_path));
        }

        let file_name = match &profile_type {
            EnvProfileType::Development => ".env.development".to_string(),
            EnvProfileType::Staging => ".env.staging".to_string(),
            EnvProfileType::Production => ".env.production".to_string(),
            EnvProfileType::Custom(name) => format!(".env.{}", name.to_lowercase().replace(' ', "-")),
        };

        let dest_path = self.project_path.join(&file_name);

        // Read the source file
        let content = fs::read_to_string(&source_path)
            .map_err(|e| format!("Failed to read import file: {}", e))?;

        // Write to the destination
        fs::write(&dest_path, content)
            .map_err(|e| format!("Failed to write profile: {}", e))?;

        Ok(file_name)
    }

    /// Compare two profiles and return the differences
    pub fn compare_profiles(
        &self,
        profile_a: &str,
        profile_b: &str,
    ) -> Result<ProfileComparison, String> {
        let vars_a = self.read_env_file(profile_a)?;
        let vars_b = self.read_env_file(profile_b)?;

        let keys_a: std::collections::HashSet<_> = vars_a.iter().map(|v| &v.key).collect();
        let keys_b: std::collections::HashSet<_> = vars_b.iter().map(|v| &v.key).collect();

        let vars_a_map: HashMap<_, _> = vars_a.iter().map(|v| (&v.key, &v.value)).collect();
        let vars_b_map: HashMap<_, _> = vars_b.iter().map(|v| (&v.key, &v.value)).collect();

        let only_in_a: Vec<String> = keys_a.difference(&keys_b).map(|k| (*k).clone()).collect();
        let only_in_b: Vec<String> = keys_b.difference(&keys_a).map(|k| (*k).clone()).collect();

        let mut different_values = Vec::new();
        for key in keys_a.intersection(&keys_b) {
            let value_a = vars_a_map.get(*key).unwrap();
            let value_b = vars_b_map.get(*key).unwrap();
            if value_a != value_b {
                different_values.push(ProfileDiff {
                    key: (*key).clone(),
                    value_a: (*value_a).clone(),
                    value_b: (*value_b).clone(),
                });
            }
        }

        Ok(ProfileComparison {
            profile_a: profile_a.to_string(),
            profile_b: profile_b.to_string(),
            only_in_a,
            only_in_b,
            different_values,
        })
    }

    /// Merge variables from one profile into another
    pub fn merge_profiles(
        &self,
        source: &str,
        target: &str,
        overwrite: bool,
    ) -> Result<Vec<EnvVariable>, String> {
        let source_vars = self.read_env_file(source)?;
        let mut target_vars = self.read_env_file(target)?;

        let target_keys: std::collections::HashSet<_> = target_vars.iter().map(|v| v.key.clone()).collect();

        for source_var in source_vars {
            if target_keys.contains(&source_var.key) {
                if overwrite {
                    if let Some(var) = target_vars.iter_mut().find(|v| v.key == source_var.key) {
                        var.value = source_var.value;
                    }
                }
            } else {
                target_vars.push(source_var);
            }
        }

        self.write_env_file(target, &target_vars)?;
        Ok(target_vars)
    }

    pub fn get_template(framework: &str) -> Vec<EnvVariable> {
        match framework {
            "nextjs" => vec![
                EnvVariable {
                    key: "NODE_ENV".to_string(),
                    value: "development".to_string(),
                    is_secret: false,
                    comment: Some("Node environment".to_string()),
                },
                EnvVariable {
                    key: "NEXT_PUBLIC_API_URL".to_string(),
                    value: "http://localhost:3000/api".to_string(),
                    is_secret: false,
                    comment: Some("Public API URL".to_string()),
                },
                EnvVariable {
                    key: "DATABASE_URL".to_string(),
                    value: "".to_string(),
                    is_secret: true,
                    comment: Some("Database connection string".to_string()),
                },
                EnvVariable {
                    key: "NEXTAUTH_SECRET".to_string(),
                    value: "".to_string(),
                    is_secret: true,
                    comment: Some("NextAuth secret".to_string()),
                },
                EnvVariable {
                    key: "NEXTAUTH_URL".to_string(),
                    value: "http://localhost:3000".to_string(),
                    is_secret: false,
                    comment: Some("NextAuth URL".to_string()),
                },
            ],
            "laravel" => vec![
                EnvVariable {
                    key: "APP_NAME".to_string(),
                    value: "Laravel".to_string(),
                    is_secret: false,
                    comment: Some("Application name".to_string()),
                },
                EnvVariable {
                    key: "APP_ENV".to_string(),
                    value: "local".to_string(),
                    is_secret: false,
                    comment: None,
                },
                EnvVariable {
                    key: "APP_KEY".to_string(),
                    value: "".to_string(),
                    is_secret: true,
                    comment: None,
                },
                EnvVariable {
                    key: "APP_DEBUG".to_string(),
                    value: "true".to_string(),
                    is_secret: false,
                    comment: None,
                },
                EnvVariable {
                    key: "APP_URL".to_string(),
                    value: "http://localhost".to_string(),
                    is_secret: false,
                    comment: None,
                },
                EnvVariable {
                    key: "DB_CONNECTION".to_string(),
                    value: "mysql".to_string(),
                    is_secret: false,
                    comment: Some("Database settings".to_string()),
                },
                EnvVariable {
                    key: "DB_HOST".to_string(),
                    value: "127.0.0.1".to_string(),
                    is_secret: false,
                    comment: None,
                },
                EnvVariable {
                    key: "DB_PORT".to_string(),
                    value: "3306".to_string(),
                    is_secret: false,
                    comment: None,
                },
                EnvVariable {
                    key: "DB_DATABASE".to_string(),
                    value: "laravel".to_string(),
                    is_secret: false,
                    comment: None,
                },
                EnvVariable {
                    key: "DB_USERNAME".to_string(),
                    value: "root".to_string(),
                    is_secret: false,
                    comment: None,
                },
                EnvVariable {
                    key: "DB_PASSWORD".to_string(),
                    value: "".to_string(),
                    is_secret: true,
                    comment: None,
                },
            ],
            "django" => vec![
                EnvVariable {
                    key: "DEBUG".to_string(),
                    value: "True".to_string(),
                    is_secret: false,
                    comment: Some("Django settings".to_string()),
                },
                EnvVariable {
                    key: "SECRET_KEY".to_string(),
                    value: "".to_string(),
                    is_secret: true,
                    comment: None,
                },
                EnvVariable {
                    key: "DATABASE_URL".to_string(),
                    value: "".to_string(),
                    is_secret: true,
                    comment: None,
                },
                EnvVariable {
                    key: "ALLOWED_HOSTS".to_string(),
                    value: "localhost,127.0.0.1".to_string(),
                    is_secret: false,
                    comment: None,
                },
            ],
            _ => vec![
                EnvVariable {
                    key: "PORT".to_string(),
                    value: "3000".to_string(),
                    is_secret: false,
                    comment: Some("Application port".to_string()),
                },
                EnvVariable {
                    key: "NODE_ENV".to_string(),
                    value: "development".to_string(),
                    is_secret: false,
                    comment: None,
                },
            ],
        }
    }
}
