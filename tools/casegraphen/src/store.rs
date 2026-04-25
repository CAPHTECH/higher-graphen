use crate::model::{
    CaseGraph, CoveragePolicy, ProjectionDefinition, CASE_GRAPH_SCHEMA, COVERAGE_POLICY_SCHEMA,
    PROJECTION_SCHEMA,
};
use crate::workflow_model::{
    WorkflowCaseGraph, WORKFLOW_GRAPH_SCHEMA, WORKFLOW_GRAPH_SCHEMA_VERSION,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug)]
pub enum StoreError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    Contract {
        path: PathBuf,
        reason: String,
    },
}

pub struct LocalCaseStore {
    root: PathBuf,
}

impl LocalCaseStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn create_graph(&self, graph: &CaseGraph) -> StoreResult<PathBuf> {
        fs::create_dir_all(&self.root).map_err(|source| StoreError::Io {
            path: self.root.clone(),
            source,
        })?;
        let path = self.path_for_graph(graph);
        write_json(&path, graph)?;
        Ok(path)
    }

    pub fn list_graphs(&self) -> StoreResult<Vec<Value>> {
        let entries = fs::read_dir(&self.root).map_err(|source| StoreError::Io {
            path: self.root.clone(),
            source,
        })?;
        let mut graphs = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| StoreError::Io {
                path: self.root.clone(),
                source,
            })?;
            collect_graph_entry(&entry.path(), &mut graphs)?;
        }
        graphs.sort_by_key(|value| value["case_graph_id"].as_str().unwrap_or("").to_owned());
        Ok(graphs)
    }

    fn path_for_graph(&self, graph: &CaseGraph) -> PathBuf {
        self.root.join(format!(
            "{}.case.graph.json",
            file_stem(&graph.case_graph_id)
        ))
    }
}

pub fn read_case_graph(path: &Path) -> StoreResult<CaseGraph> {
    let graph: CaseGraph = read_json(path)?;
    require_schema(path, &graph.schema, CASE_GRAPH_SCHEMA)?;
    Ok(graph)
}

pub fn read_coverage_policy(path: &Path) -> StoreResult<CoveragePolicy> {
    let policy: CoveragePolicy = read_json(path)?;
    require_schema(path, &policy.schema, COVERAGE_POLICY_SCHEMA)?;
    Ok(policy)
}

pub fn read_projection(path: &Path) -> StoreResult<ProjectionDefinition> {
    let projection: ProjectionDefinition = read_json(path)?;
    require_schema(path, &projection.schema, PROJECTION_SCHEMA)?;
    Ok(projection)
}

pub fn read_workflow_graph(path: &Path) -> StoreResult<WorkflowCaseGraph> {
    let graph: WorkflowCaseGraph = read_json(path)?;
    require_schema(path, &graph.schema, WORKFLOW_GRAPH_SCHEMA)?;
    require_schema_version(path, graph.schema_version, WORKFLOW_GRAPH_SCHEMA_VERSION)?;
    Ok(graph)
}

pub fn write_report(path: &Path, report: &impl serde::Serialize) -> StoreResult<()> {
    write_json(path, report)
}

fn collect_graph_entry(path: &Path, graphs: &mut Vec<Value>) -> StoreResult<()> {
    if !is_graph_file(path) {
        return Ok(());
    }
    let graph = read_case_graph(path)?;
    graphs.push(json!({
        "case_graph_id": graph.case_graph_id,
        "space_id": graph.space_id,
        "path": path.display().to_string(),
        "counts": crate::eval::graph_counts(&graph)
    }));
    Ok(())
}

fn read_json<T: DeserializeOwned>(path: &Path) -> StoreResult<T> {
    let text = fs::read_to_string(path).map_err(|source| StoreError::Io {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| StoreError::Json {
        path: path.to_owned(),
        source,
    })
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> StoreResult<()> {
    let text = serde_json::to_string_pretty(value).map_err(|source| StoreError::Json {
        path: path.to_owned(),
        source,
    })?;
    fs::write(path, format!("{text}\n")).map_err(|source| StoreError::Io {
        path: path.to_owned(),
        source,
    })
}

fn require_schema(path: &Path, actual: &str, expected: &str) -> StoreResult<()> {
    if actual == expected {
        return Ok(());
    }
    Err(StoreError::Contract {
        path: path.to_owned(),
        reason: format!("unsupported schema {actual:?}; expected {expected:?}"),
    })
}

fn require_schema_version(path: &Path, actual: u32, expected: u32) -> StoreResult<()> {
    if actual == expected {
        return Ok(());
    }
    Err(StoreError::Contract {
        path: path.to_owned(),
        reason: format!("unsupported schema version {actual}; expected {expected}"),
    })
}

fn is_graph_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".case.graph.json"))
}

fn file_stem(id: &higher_graphen_core::Id) -> String {
    id.as_str()
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' => character,
            _ => '-',
        })
        .collect()
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Json { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Contract { path, reason } => write!(formatter, "{}: {reason}", path.display()),
        }
    }
}

impl std::error::Error for StoreError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::sample_graph;

    #[test]
    fn local_store_round_trips_graph_headers() {
        let root =
            std::env::temp_dir().join(format!("casegraphen-store-test-{}", std::process::id()));
        let store = LocalCaseStore::new(root.clone());
        let path = store.create_graph(&sample_graph()).expect("create graph");
        let entries = store.list_graphs().expect("list graphs");

        assert_eq!(
            entries[0]["case_graph_id"],
            json!("case_graph:architecture-smoke")
        );
        assert!(path.exists());

        fs::remove_dir_all(root).expect("remove temp store");
    }

    #[test]
    fn read_workflow_graph_rejects_unsupported_schema() {
        let root = std::env::temp_dir().join(format!(
            "casegraphen-workflow-store-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create temp store");
        let path = root.join("bad.workflow.graph.json");
        let mut value: Value = serde_json::from_str(include_str!(
            "../../../schemas/casegraphen/workflow.graph.example.json"
        ))
        .expect("workflow graph example");
        value["schema"] = json!("highergraphen.case.workflow.graph.v0");
        write_json(&path, &value).expect("write workflow graph");

        let error = read_workflow_graph(&path).expect_err("unsupported schema");
        assert!(error.to_string().contains("unsupported schema"));

        fs::remove_dir_all(root).expect("remove temp store");
    }
}
