use crate::config::Config;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum, Serialize, Deserialize)]
pub enum OutputFormat {
    Tree,
    Files,
    Both,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TokenizerType {
    Tiktoken,
    #[clap(name = "huggingface")]
    HuggingFace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Exclude {
    File(PathBuf),
    Pattern(String),
}

#[derive(Parser, Debug)]
#[command(
    name = "glimpse",
    about = "A blazingly fast tool for peeking at codebases",
    version
)]
pub struct Cli {
    /// Files or directories to analyze
    #[arg(value_parser = validate_path, default_value = ".")]
    pub paths: Vec<PathBuf>,

    /// Print the config file path and exit
    #[arg(long)]
    pub config_path: bool,

    /// Additional patterns to include (e.g. "*.rs,*.go")
    #[arg(short, long, value_delimiter = ',')]
    pub include: Option<Vec<String>>,

    /// Additional patterns to exclude
    #[arg(short, long, value_parser = parse_exclude, value_delimiter = ',')]
    pub exclude: Option<Vec<Exclude>>,

    /// Maximum file size in bytes
    #[arg(short, long)]
    pub max_size: Option<u64>,

    /// Maximum directory depth
    #[arg(long)]
    pub max_depth: Option<usize>,

    /// Output format (tree, files, or both)
    #[arg(short, long, value_enum)]
    pub output: Option<OutputFormat>,

    /// Output file path (optional)
    #[arg(short = 'f', long)]
    pub file: Option<PathBuf>,

    /// Print to stdout instead
    #[arg(short, long)]
    pub print: bool,

    /// Number of threads for parallel processing
    #[arg(short, long)]
    pub threads: Option<usize>,

    /// Show hidden files and directories
    #[arg(short = 'H', long)]
    pub hidden: bool,

    /// Don't respect .gitignore files
    #[arg(long)]
    pub no_ignore: bool,

    /// Ignore Token Count
    #[arg(long)]
    pub no_tokens: bool,

    /// Tokenizer to use (tiktoken or huggingface)
    #[arg(long, value_enum)]
    pub tokenizer: Option<TokenizerType>,

    /// Model to use for HuggingFace tokenizer
    #[arg(long)]
    pub model: Option<String>,

    /// Path to local tokenizer file
    #[arg(long)]
    pub tokenizer_file: Option<PathBuf>,

    /// Interactive mode
    #[arg(long)]
    pub interactive: bool,

    /// Output as Pdf
    #[arg(long)]
    pub pdf: Option<PathBuf>,
}

impl Cli {
    pub fn parse_with_config(config: &Config) -> anyhow::Result<Self> {
        let mut cli = Self::parse();

        // Apply config defaults if CLI args aren't specified
        cli.max_size = cli.max_size.or(Some(config.max_size));
        cli.max_depth = cli.max_depth.or(Some(config.max_depth));
        cli.output = cli.output.or(Some(OutputFormat::from(
            config.default_output_format.clone(),
        )));

        // Merge excludes from config and CLI
        if let Some(mut excludes) = cli.exclude.take() {
            excludes.extend(config.default_excludes.clone());
            cli.exclude = Some(excludes);
        } else {
            cli.exclude = Some(config.default_excludes.clone());
        }

        // Set default tokenizer if none specified but token counting is enabled
        if !cli.no_tokens && cli.tokenizer.is_none() {
            cli.tokenizer = Some(match config.default_tokenizer.as_str() {
                "huggingface" => TokenizerType::HuggingFace,
                _ => TokenizerType::Tiktoken,
            });
        }

        // Set default model for HuggingFace if none specified
        if cli
            .tokenizer
            .as_ref()
            .is_some_and(|t| matches!(t, TokenizerType::HuggingFace))
            && cli.model.is_none()
            && cli.tokenizer_file.is_none()
        {
            cli.model = Some(config.default_tokenizer_model.clone());
        }

        Ok(cli)
    }
}

fn validate_path(path: &str) -> Result<PathBuf, String> {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        return Err(format!("Path '{}' does not exist", path));
    }
    Ok(path_buf)
}

fn parse_exclude(value: &str) -> Result<Exclude, String> {
    let path = PathBuf::from(value);
    if path.exists() {
        Ok(Exclude::File(path))
    } else {
        Ok(Exclude::Pattern(value.to_string()))
    }
}
