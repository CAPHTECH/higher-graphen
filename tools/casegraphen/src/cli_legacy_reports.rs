use super::{serialize, CliError};
use crate::{model::CaseGraph, report, store::LocalCaseStore};
use higher_graphen_core::Id;
use std::path::Path;

pub(super) fn run_create(
    case_graph_id: &str,
    space_id: &str,
    store: &Path,
) -> Result<String, CliError> {
    let graph = CaseGraph::empty(
        Id::new(case_graph_id.to_owned())?,
        Id::new(space_id.to_owned())?,
    );
    let path = LocalCaseStore::new(store.to_path_buf()).create_graph(&graph)?;
    serialize(&report::create_report("casegraphen create", &path, &graph))
}
