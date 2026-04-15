// Source: /data/home/swei/claudecode/openclaudecode/src/types/notebook.ts
//! Jupyter notebook utilities.

use serde::{Deserialize, Serialize};

/// A Jupyter notebook cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookCell {
    pub cell_type: String,
    pub source: Vec<String>,
    pub metadata: serde_json::Value,
    pub outputs: Option<Vec<serde_json::Value>>,
    pub execution_count: Option<u32>,
}

/// A Jupyter notebook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub nbformat: u32,
    pub nbformat_minor: u32,
    pub metadata: NotebookMetadata,
    pub cells: Vec<NotebookCell>,
}

/// Notebook metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookMetadata {
    pub kernelspec: Option<KernelSpec>,
    pub language_info: Option<LanguageInfo>,
}

/// Kernel specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelSpec {
    pub name: String,
    pub display_name: String,
}

/// Language info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub name: String,
    pub version: Option<String>,
}

/// Check if a file is a Jupyter notebook
pub fn is_notebook_file(path: &str) -> bool {
    path.ends_with(".ipynb")
}

/// Parse a notebook from JSON
pub fn parse_notebook(json: &str) -> Result<Notebook, serde_json::Error> {
    serde_json::from_str(json)
}

/// Extract code from notebook cells
pub fn extract_code_cells(notebook: &Notebook) -> Vec<String> {
    notebook
        .cells
        .iter()
        .filter(|c| c.cell_type == "code")
        .map(|c| c.source.join(""))
        .collect()
}
