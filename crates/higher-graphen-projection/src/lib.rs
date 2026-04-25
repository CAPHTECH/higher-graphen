//! Projection definitions, selectors, results, and renderers for HigherGraphen.

use higher_graphen_core::{CoreError, Id, Result, Severity};
use serde::{Deserialize, Serialize};

/// Target consumer for a projected view.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionAudience {
    /// A human reader.
    Human,
    /// An AI agent or model.
    Ai,
    /// A software developer.
    Developer,
    /// An architecture or design reviewer.
    Architect,
    /// An executive or strategy stakeholder.
    Executive,
    /// An operations or reliability stakeholder.
    Operator,
    /// Another system consuming projection data.
    ExternalSystem,
}

/// Intended use of a projected view.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionPurpose {
    /// Explain selected structure.
    Explanation,
    /// Produce a report.
    Report,
    /// Produce dashboard-ready data without UI concerns.
    Dashboard,
    /// Produce an action plan.
    ActionPlan,
    /// Support a review workflow.
    Review,
    /// Return selected query results.
    QueryResult,
    /// Produce data for an API layer without encoding transport details.
    ApiResponse,
}

/// Transport-neutral output shape for a projection.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OutputSchema {
    /// Unstructured text output.
    Text,
    /// Sectioned output.
    Sections {
        /// Stable section names expected in the output.
        section_names: Vec<String>,
    },
    /// Tabular output.
    Table {
        /// Stable column names expected in each row.
        columns: Vec<String>,
    },
    /// Key-value output.
    KeyValue {
        /// Stable keys expected in the output.
        keys: Vec<String>,
    },
    /// Downstream-owned schema name and fields.
    Custom {
        /// Stable schema name.
        name: String,
        /// Stable field names in the custom schema.
        fields: Vec<String>,
    },
}

impl OutputSchema {
    /// Creates a text output schema.
    pub fn text() -> Self {
        Self::Text
    }

    /// Creates a sectioned output schema with non-empty section names.
    pub fn sections<I, S>(section_names: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Ok(Self::Sections {
            section_names: collect_non_empty_texts(section_names, "section_names")?,
        })
    }

    /// Creates a tabular output schema with non-empty column names.
    pub fn table<I, S>(columns: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Ok(Self::Table {
            columns: collect_non_empty_texts(columns, "columns")?,
        })
    }

    /// Creates a key-value output schema with non-empty keys.
    pub fn key_value<I, S>(keys: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Ok(Self::KeyValue {
            keys: collect_non_empty_texts(keys, "keys")?,
        })
    }

    /// Creates a custom output schema with a non-empty name and field list.
    pub fn custom<I, S>(name: impl Into<String>, fields: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Ok(Self::Custom {
            name: normalized_text(name, "name")?,
            fields: collect_non_empty_texts(fields, "fields")?,
        })
    }
}

/// Declared information loss caused by a projection.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InformationLoss {
    /// Human-readable description of what was omitted, summarized, or simplified.
    pub description: String,
    /// Source structure identifiers affected by this loss declaration.
    pub source_ids: Vec<Id>,
}

impl InformationLoss {
    /// Creates a loss declaration with explicit source identifiers.
    pub fn declared<I>(description: impl Into<String>, source_ids: I) -> Result<Self>
    where
        I: IntoIterator<Item = Id>,
    {
        Ok(Self {
            description: normalized_text(description, "description")?,
            source_ids: collect_non_empty_ids(source_ids, "source_ids")?,
        })
    }

    /// Returns the source identifiers affected by this loss declaration.
    pub fn source_ids(&self) -> &[Id] {
        &self.source_ids
    }
}

/// Transport-neutral renderer category used to produce projection output.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RendererKind {
    /// Plain text renderer.
    PlainText,
    /// Markdown renderer.
    Markdown,
    /// Table renderer.
    Table,
    /// Structured data renderer.
    Structured,
    /// Downstream-owned renderer.
    Custom(String),
}

impl RendererKind {
    /// Creates a custom renderer kind with a non-empty name.
    pub fn custom(name: impl Into<String>) -> Result<Self> {
        Ok(Self::Custom(normalized_text(name, "name")?))
    }
}

/// Selector describing which source structures a projection reads.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionSelector {
    /// Selected cell identifiers. Empty means no cell-id filter.
    pub cell_ids: Vec<Id>,
    /// Selected cell type identifiers. Empty means no cell-type filter.
    pub cell_type_ids: Vec<Id>,
    /// Selected obstruction identifiers. Empty means no obstruction-id filter.
    pub obstruction_ids: Vec<Id>,
    /// Selected obstruction type identifiers. Empty means no obstruction-type filter.
    pub obstruction_type_ids: Vec<Id>,
    /// Selected context identifiers. Empty means no context filter.
    pub context_ids: Vec<Id>,
    /// Minimum severity for selected structures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_severity: Option<Severity>,
}

impl ProjectionSelector {
    /// Creates a selector with no filters.
    pub fn all() -> Self {
        Self::default()
    }

    /// Returns this selector with explicit cell identifiers.
    pub fn with_cell_ids<I>(mut self, cell_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.cell_ids = cell_ids.into_iter().collect();
        self
    }

    /// Returns this selector with explicit cell type identifiers.
    pub fn with_cell_type_ids<I>(mut self, cell_type_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.cell_type_ids = cell_type_ids.into_iter().collect();
        self
    }

    /// Returns this selector with explicit obstruction identifiers.
    pub fn with_obstruction_ids<I>(mut self, obstruction_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.obstruction_ids = obstruction_ids.into_iter().collect();
        self
    }

    /// Returns this selector with explicit obstruction type identifiers.
    pub fn with_obstruction_type_ids<I>(mut self, obstruction_type_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.obstruction_type_ids = obstruction_type_ids.into_iter().collect();
        self
    }

    /// Returns this selector with explicit context identifiers.
    pub fn with_context_ids<I>(mut self, context_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.context_ids = context_ids.into_iter().collect();
        self
    }

    /// Returns this selector with a minimum severity filter.
    pub fn with_min_severity(mut self, min_severity: Severity) -> Self {
        self.min_severity = Some(min_severity);
        self
    }
}

/// Projection definition connecting source structure to an audience-specific view.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Projection {
    /// Projection identifier.
    pub id: Id,
    /// Source space identifier.
    pub source_space_id: Id,
    /// Human-readable projection name.
    pub name: String,
    /// Target audience.
    pub audience: ProjectionAudience,
    /// Projection purpose.
    pub purpose: ProjectionPurpose,
    /// Source structure selector.
    pub input_selector: ProjectionSelector,
    /// Expected output shape.
    pub output_schema: OutputSchema,
    /// Declared information loss for this projection.
    pub information_loss: Vec<InformationLoss>,
    /// Optional transport-neutral renderer choice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renderer: Option<RendererKind>,
}

impl Projection {
    /// Creates a projection with explicit information-loss declarations.
    #[allow(clippy::too_many_arguments)]
    pub fn new<I>(
        id: Id,
        source_space_id: Id,
        name: impl Into<String>,
        audience: ProjectionAudience,
        purpose: ProjectionPurpose,
        input_selector: ProjectionSelector,
        output_schema: OutputSchema,
        information_loss: I,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = InformationLoss>,
    {
        Ok(Self {
            id,
            source_space_id,
            name: normalized_text(name, "name")?,
            audience,
            purpose,
            input_selector,
            output_schema,
            information_loss: collect_non_empty_information_loss(information_loss)?,
            renderer: None,
        })
    }

    /// Returns this projection with a renderer choice.
    pub fn with_renderer(mut self, renderer: RendererKind) -> Self {
        self.renderer = Some(renderer);
        self
    }

    /// Returns the declared information-loss entries.
    pub fn information_loss(&self) -> &[InformationLoss] {
        &self.information_loss
    }
}

/// Transport-neutral projected output.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProjectionOutput {
    /// Plain text output.
    Text {
        /// Rendered text.
        text: String,
    },
    /// Sectioned output.
    Sections {
        /// Rendered sections.
        sections: Vec<ProjectionSection>,
    },
    /// Tabular output.
    Table {
        /// Stable column names.
        columns: Vec<String>,
        /// Rendered rows aligned to `columns`.
        rows: Vec<Vec<String>>,
    },
    /// Key-value output.
    KeyValue {
        /// Rendered entries.
        entries: Vec<ProjectionEntry>,
    },
}

impl ProjectionOutput {
    /// Creates plain text output.
    pub fn text(text: impl Into<String>) -> Result<Self> {
        Ok(Self::Text {
            text: normalized_text(text, "text")?,
        })
    }

    /// Creates sectioned output.
    pub fn sections<I>(sections: I) -> Result<Self>
    where
        I: IntoIterator<Item = ProjectionSection>,
    {
        Ok(Self::Sections {
            sections: collect_non_empty_items(sections, "sections")?,
        })
    }

    /// Creates table output.
    pub fn table<I, S>(columns: I, rows: Vec<Vec<String>>) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let columns = collect_non_empty_texts(columns, "columns")?;
        for row in &rows {
            if row.len() != columns.len() {
                return Err(malformed_field(
                    "rows",
                    "each row must have the same number of values as columns",
                ));
            }
        }

        Ok(Self::Table { columns, rows })
    }

    /// Creates key-value output.
    pub fn key_value<I>(entries: I) -> Result<Self>
    where
        I: IntoIterator<Item = ProjectionEntry>,
    {
        Ok(Self::KeyValue {
            entries: collect_non_empty_items(entries, "entries")?,
        })
    }
}

/// A section in a projected output.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionSection {
    /// Section title.
    pub title: String,
    /// Section body.
    pub body: String,
    /// Source identifiers represented by this section.
    pub source_ids: Vec<Id>,
}

impl ProjectionSection {
    /// Creates a section with explicit source identifiers.
    pub fn new<I>(title: impl Into<String>, body: impl Into<String>, source_ids: I) -> Result<Self>
    where
        I: IntoIterator<Item = Id>,
    {
        Ok(Self {
            title: normalized_text(title, "title")?,
            body: normalized_text(body, "body")?,
            source_ids: collect_non_empty_ids(source_ids, "source_ids")?,
        })
    }

    /// Returns the source identifiers represented by this section.
    pub fn source_ids(&self) -> &[Id] {
        &self.source_ids
    }
}

/// A key-value entry in a projected output.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionEntry {
    /// Entry key.
    pub key: String,
    /// Entry value.
    pub value: String,
    /// Source identifiers represented by this entry.
    pub source_ids: Vec<Id>,
}

impl ProjectionEntry {
    /// Creates an entry with explicit source identifiers.
    pub fn new<I>(key: impl Into<String>, value: impl Into<String>, source_ids: I) -> Result<Self>
    where
        I: IntoIterator<Item = Id>,
    {
        Ok(Self {
            key: normalized_text(key, "key")?,
            value: normalized_text(value, "value")?,
            source_ids: collect_non_empty_ids(source_ids, "source_ids")?,
        })
    }

    /// Returns the source identifiers represented by this entry.
    pub fn source_ids(&self) -> &[Id] {
        &self.source_ids
    }
}

/// Result of applying a projection.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionResult {
    /// Projection identifier used to produce this result.
    pub projection_id: Id,
    /// Target audience copied from the projection definition.
    pub audience: ProjectionAudience,
    /// Purpose copied from the projection definition.
    pub purpose: ProjectionPurpose,
    /// Output schema used for this result.
    pub output_schema: OutputSchema,
    /// Renderer used to produce the output.
    pub renderer: RendererKind,
    /// Transport-neutral rendered output.
    pub output: ProjectionOutput,
    /// Source identifiers actually represented in the output.
    pub source_ids: Vec<Id>,
    /// Information-loss declarations that apply to the output.
    pub information_loss: Vec<InformationLoss>,
}

impl ProjectionResult {
    /// Creates a result with explicit source identifiers and information loss.
    #[allow(clippy::too_many_arguments)]
    pub fn new<I, L>(
        projection_id: Id,
        audience: ProjectionAudience,
        purpose: ProjectionPurpose,
        output_schema: OutputSchema,
        renderer: RendererKind,
        output: ProjectionOutput,
        source_ids: I,
        information_loss: L,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = Id>,
        L: IntoIterator<Item = InformationLoss>,
    {
        Ok(Self {
            projection_id,
            audience,
            purpose,
            output_schema,
            renderer,
            output,
            source_ids: collect_non_empty_ids(source_ids, "source_ids")?,
            information_loss: collect_non_empty_information_loss(information_loss)?,
        })
    }

    /// Creates a result from a projection while still requiring explicit trace data.
    pub fn from_projection<I, L>(
        projection: &Projection,
        renderer: RendererKind,
        output: ProjectionOutput,
        source_ids: I,
        information_loss: L,
    ) -> Result<Self>
    where
        I: IntoIterator<Item = Id>,
        L: IntoIterator<Item = InformationLoss>,
    {
        Self::new(
            projection.id.clone(),
            projection.audience,
            projection.purpose,
            projection.output_schema.clone(),
            renderer,
            output,
            source_ids,
            information_loss,
        )
    }

    /// Returns the source identifiers actually represented in the output.
    pub fn source_ids(&self) -> &[Id] {
        &self.source_ids
    }

    /// Returns the information-loss declarations that apply to the output.
    pub fn information_loss(&self) -> &[InformationLoss] {
        &self.information_loss
    }
}

fn normalized_text(value: impl Into<String>, field: &'static str) -> Result<String> {
    let raw = value.into();
    let normalized = raw.trim().to_owned();

    if normalized.is_empty() {
        return Err(malformed_field(
            field,
            "value must not be empty after trimming",
        ));
    }

    Ok(normalized)
}

fn collect_non_empty_texts<I, S>(values: I, field: &'static str) -> Result<Vec<String>>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let texts = values
        .into_iter()
        .map(|value| normalized_text(value, field))
        .collect::<Result<Vec<_>>>()?;

    if texts.is_empty() {
        return Err(malformed_field(field, "at least one value is required"));
    }

    Ok(texts)
}

fn collect_non_empty_ids<I>(ids: I, field: &'static str) -> Result<Vec<Id>>
where
    I: IntoIterator<Item = Id>,
{
    let ids = ids.into_iter().collect::<Vec<_>>();

    if ids.is_empty() {
        return Err(malformed_field(field, "at least one id is required"));
    }

    Ok(ids)
}

fn collect_non_empty_items<I, T>(items: I, field: &'static str) -> Result<Vec<T>>
where
    I: IntoIterator<Item = T>,
{
    let items = items.into_iter().collect::<Vec<_>>();

    if items.is_empty() {
        return Err(malformed_field(field, "at least one value is required"));
    }

    Ok(items)
}

fn collect_non_empty_information_loss<I>(information_loss: I) -> Result<Vec<InformationLoss>>
where
    I: IntoIterator<Item = InformationLoss>,
{
    collect_non_empty_items(information_loss, "information_loss")
}

fn malformed_field(field: impl Into<String>, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Id {
        Id::new(value).expect("test id should be valid")
    }

    fn loss(source_id: &str) -> InformationLoss {
        InformationLoss::declared("summarized detail", [id(source_id)])
            .expect("test loss should be valid")
    }

    #[test]
    fn projection_requires_declared_information_loss() {
        let projection = Projection::new(
            id("projection:architecture-summary"),
            id("space:architecture"),
            "Architecture summary",
            ProjectionAudience::Architect,
            ProjectionPurpose::Report,
            ProjectionSelector::all(),
            OutputSchema::sections(["summary", "risks"]).expect("schema should be valid"),
            Vec::<InformationLoss>::new(),
        );

        assert_eq!(
            projection
                .expect_err("empty information loss should fail")
                .code(),
            "malformed_field"
        );
    }

    #[test]
    fn result_requires_explicit_source_ids() {
        let result = ProjectionResult::new(
            id("projection:architecture-summary"),
            ProjectionAudience::Architect,
            ProjectionPurpose::Report,
            OutputSchema::text(),
            RendererKind::PlainText,
            ProjectionOutput::text("summary").expect("output should be valid"),
            Vec::<Id>::new(),
            [loss("cell:service-a")],
        );

        assert_eq!(
            result.expect_err("empty source ids should fail").code(),
            "malformed_field"
        );
    }

    #[test]
    fn output_sections_keep_section_source_ids() {
        let source_id = id("cell:service-a");
        let section = ProjectionSection::new("Risk", "Dependency is unstable", [source_id.clone()])
            .expect("section should be valid");

        let output = ProjectionOutput::sections([section.clone()]).expect("output should be valid");

        assert_eq!(section.source_ids(), [source_id]);
        assert!(matches!(output, ProjectionOutput::Sections { .. }));
    }

    #[test]
    fn projection_result_can_be_created_from_projection_with_explicit_trace_data() {
        let source_id = id("cell:service-a");
        let declared_loss = loss(source_id.as_str());
        let projection = Projection::new(
            id("projection:architecture-summary"),
            id("space:architecture"),
            " Architecture summary ",
            ProjectionAudience::Architect,
            ProjectionPurpose::Report,
            ProjectionSelector::all().with_cell_ids([source_id.clone()]),
            OutputSchema::key_value(["risk"]).expect("schema should be valid"),
            [declared_loss.clone()],
        )
        .expect("projection should be valid")
        .with_renderer(RendererKind::Structured);

        let result = ProjectionResult::from_projection(
            &projection,
            RendererKind::Structured,
            ProjectionOutput::key_value([ProjectionEntry::new(
                "risk",
                "Dependency is unstable",
                [source_id.clone()],
            )
            .expect("entry should be valid")])
            .expect("output should be valid"),
            [source_id.clone()],
            [declared_loss],
        )
        .expect("result should be valid");

        assert_eq!(projection.name, "Architecture summary");
        assert_eq!(result.source_ids(), [source_id]);
        assert_eq!(result.information_loss().len(), 1);
    }

    #[test]
    fn table_output_requires_rows_to_match_columns() {
        let output = ProjectionOutput::table(["cell", "risk"], vec![vec!["cell:1".to_owned()]]);

        assert_eq!(
            output.expect_err("mismatched row should fail").code(),
            "malformed_field"
        );
    }
}
