use super::{
    ops::NativeCloseGateOptions,
    options::{required_segment, NativeOptions},
    NativeCliCommand, NativeCliError, NativeReasonSection,
};
use std::ffi::OsString;

impl NativeCliCommand {
    pub fn parse(
        namespace: &str,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let mut args = args.into_iter();
        match namespace {
            "case" => Self::parse_case(required_segment(&mut args, "case operation")?, args),
            "space" => Self::parse_space(required_segment(&mut args, "space operation")?, args),
            "lift" => Self::parse_lift(required_segment(&mut args, "lift adapter")?, args),
            "obstruction" => {
                Self::parse_obstruction(required_segment(&mut args, "obstruction operation")?, args)
            }
            "completion" => {
                Self::parse_completion(required_segment(&mut args, "completion operation")?, args)
            }
            "projection" => {
                Self::parse_projection(required_segment(&mut args, "projection operation")?, args)
            }
            "equivalence" => {
                Self::parse_equivalence(required_segment(&mut args, "equivalence operation")?, args)
            }
            "invariant" => {
                Self::parse_invariant(required_segment(&mut args, "invariant operation")?, args)
            }
            "morphism" => {
                Self::parse_morphism(required_segment(&mut args, "morphism operation")?, args)
            }
            _ => Err(NativeCliError::usage("unsupported native namespace")),
        }
    }

    fn parse_case(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let operation = operation
            .to_str()
            .ok_or_else(|| NativeCliError::usage("case operation must be UTF-8"))?;
        let mut args = args.into_iter().collect::<Vec<_>>();
        let history_topology = is_history_topology(operation, &args);
        if history_topology {
            args.remove(0);
        }
        let history_topology_diff = history_topology
            && args
                .first()
                .and_then(|argument| argument.to_str())
                .is_some_and(|argument| argument == "diff");
        if history_topology_diff {
            args.remove(0);
        }
        let options = NativeOptions::parse(args)?;
        match operation {
            "new" | "create" => Ok(Self::CaseNew {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                space_id: options.require_id("--space-id")?,
                title: options.require_string("--title")?,
                revision_id: options.require_id("--revision-id")?,
                output: options.output,
            }),
            "import" => Ok(Self::CaseImport {
                store: options.require_store()?,
                input: options.require_path("--input")?,
                revision_id: options.require_id("--revision-id")?,
                output: options.output,
            }),
            "list" => Ok(Self::CaseList {
                store: options.require_store()?,
                output: options.output,
            }),
            "inspect" => Ok(Self::CaseInspect {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "history" => Self::parse_history_case(options, history_topology, history_topology_diff),
            "replay" => Ok(Self::CaseReplay {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "validate" => Ok(Self::CaseValidate {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "reason" => Self::parse_reason(options, NativeReasonSection::Reason),
            "frontier" => Self::parse_reason(options, NativeReasonSection::Frontier),
            "obstructions" => Self::parse_reason(options, NativeReasonSection::Obstructions),
            "completions" => Self::parse_reason(options, NativeReasonSection::Completions),
            "evidence" => Self::parse_reason(options, NativeReasonSection::Evidence),
            "project" => Self::parse_reason(options, NativeReasonSection::Project),
            "close-check" => Self::parse_close_check(options),
            _ => Err(NativeCliError::usage("unsupported native case command")),
        }
    }

    fn parse_space(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let operation = operation
            .to_str()
            .ok_or_else(|| NativeCliError::usage("space operation must be UTF-8"))?;
        let mut args = args.into_iter().collect::<Vec<_>>();
        let topology = operation == "topology";
        let topology_diff = topology
            && args
                .first()
                .and_then(|argument| argument.to_str())
                .is_some_and(|argument| argument == "diff");
        if topology_diff {
            args.remove(0);
        }
        let options = NativeOptions::parse(args)?;
        match operation {
            "new" | "create" => Ok(Self::CaseNew {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                space_id: options.require_id("--space-id")?,
                title: options.require_string("--title")?,
                revision_id: options.require_id("--revision-id")?,
                output: options.output,
            }),
            "import" => Ok(Self::CaseImport {
                store: options.require_store()?,
                input: options.require_path("--input")?,
                revision_id: options.require_id("--revision-id")?,
                output: options.output,
            }),
            "list" => Ok(Self::CaseList {
                store: options.require_store()?,
                output: options.output,
            }),
            "inspect" => Ok(Self::CaseInspect {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "history" => Ok(Self::CaseHistory {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "replay" => Ok(Self::CaseReplay {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "validate" => Ok(Self::CaseValidate {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            "reason" => Self::parse_reason(options, NativeReasonSection::Reason),
            "frontier" => Self::parse_reason(options, NativeReasonSection::Frontier),
            "topology" if topology_diff => Ok(Self::CaseTopologyDiff {
                left_store: options.require_path("--left-store")?,
                left_case_space_id: options.require_id("--left-case-space-id")?,
                right_store: options.require_path("--right-store")?,
                right_case_space_id: options.require_id("--right-case-space-id")?,
                topology_options: options.topology_options(),
                output: options.output,
            }),
            "topology" => Ok(Self::CaseTopology {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                topology_options: options.topology_options(),
                output: options.output,
            }),
            _ => Err(NativeCliError::usage("unsupported native space command")),
        }
    }

    fn parse_lift(
        adapter: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let adapter = adapter
            .to_str()
            .ok_or_else(|| NativeCliError::usage("lift adapter must be UTF-8"))?;
        let options = NativeOptions::parse(args)?;
        match adapter {
            "native" => Ok(Self::CaseImport {
                store: options.require_store()?,
                input: options.require_path("--input")?,
                revision_id: options.require_id("--revision-id")?,
                output: options.output,
            }),
            "workflow" | "case-graph" => Ok(Self::LiftStructuredSource {
                store: options.require_store()?,
                input: options.require_path("--input")?,
                revision_id: options.require_id("--revision-id")?,
                adapter: adapter.to_owned(),
                output: options.output,
            }),
            _ => Err(NativeCliError::usage("unsupported lift adapter")),
        }
    }

    fn parse_obstruction(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        match operation.to_str() {
            Some("list") => Self::parse_reason(
                NativeOptions::parse(args)?,
                NativeReasonSection::Obstructions,
            ),
            Some(_) | None => Err(NativeCliError::usage("unsupported obstruction command")),
        }
    }

    fn parse_completion(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        match operation.to_str() {
            Some("candidates") => Self::parse_reason(
                NativeOptions::parse(args)?,
                NativeReasonSection::Completions,
            ),
            Some(_) | None => Err(NativeCliError::usage("unsupported completion command")),
        }
    }

    fn parse_projection(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let options = NativeOptions::parse(args)?;
        match operation.to_str() {
            Some("apply") => {
                let projection = options.require_path("--projection")?;
                Ok(Self::ProjectionApply {
                    store: options.require_store()?,
                    case_space_id: options.require_id("--case-space-id")?,
                    projection,
                    output: options.output,
                })
            }
            Some(_) | None => Err(NativeCliError::usage("unsupported projection command")),
        }
    }

    fn parse_equivalence(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let options = NativeOptions::parse(args)?;
        match operation.to_str() {
            Some("check") => Ok(Self::EquivalenceCheck {
                left_store: options.require_path("--left-store")?,
                left_case_space_id: options.require_id("--left-case-space-id")?,
                right_store: options.require_path("--right-store")?,
                right_case_space_id: options.require_id("--right-case-space-id")?,
                topology_options: options.topology_options(),
                output: options.output,
            }),
            Some(_) | None => Err(NativeCliError::usage("unsupported equivalence command")),
        }
    }

    fn parse_invariant(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let options = NativeOptions::parse(args)?;
        match operation.to_str() {
            Some("check") => Ok(Self::InvariantCheck {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                output: options.output,
            }),
            Some("close-check") => Self::parse_close_check(options),
            Some(_) | None => Err(NativeCliError::usage("unsupported invariant command")),
        }
    }

    fn parse_close_check(options: NativeOptions) -> Result<Self, NativeCliError> {
        Ok(Self::CaseCloseCheck {
            store: options.require_store()?,
            case_space_id: options.require_id("--case-space-id")?,
            base_revision_id: options
                .base_revision_id
                .clone()
                .or(options.revision_id.clone())
                .ok_or_else(|| NativeCliError::usage("--base-revision-id <id> is required"))?,
            validation_evidence_ids: options.validation_evidence_ids,
            gate_options: NativeCloseGateOptions {
                close_policy_id: options.close_policy_id,
                actor_id: options.actor_id,
                capability_ids: options.capability_ids,
                operation_scope_id: options.operation_scope_id,
                audience: options.audience,
                source_boundary_id: options.source_boundary_id,
            },
            output: options.output,
        })
    }

    fn parse_history_case(
        options: NativeOptions,
        history_topology: bool,
        history_topology_diff: bool,
    ) -> Result<Self, NativeCliError> {
        if history_topology_diff {
            return Ok(Self::CaseTopologyDiff {
                left_store: options.require_path("--left-store")?,
                left_case_space_id: options.require_id("--left-case-space-id")?,
                right_store: options.require_path("--right-store")?,
                right_case_space_id: options.require_id("--right-case-space-id")?,
                topology_options: options.topology_options(),
                output: options.output,
            });
        }
        if history_topology {
            return Ok(Self::CaseTopology {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                topology_options: options.topology_options(),
                output: options.output,
            });
        }
        Ok(Self::CaseHistory {
            store: options.require_store()?,
            case_space_id: options.require_id("--case-space-id")?,
            output: options.output,
        })
    }

    fn parse_reason(
        options: NativeOptions,
        section: NativeReasonSection,
    ) -> Result<Self, NativeCliError> {
        Ok(Self::CaseReason {
            store: options.require_store()?,
            case_space_id: options.require_id("--case-space-id")?,
            section,
            output: options.output,
        })
    }

    fn parse_morphism(
        operation: OsString,
        args: impl IntoIterator<Item = OsString>,
    ) -> Result<Self, NativeCliError> {
        let operation = operation
            .to_str()
            .ok_or_else(|| NativeCliError::usage("morphism operation must be UTF-8"))?;
        let options = NativeOptions::parse(args)?;
        match operation {
            "propose" => Ok(Self::MorphismPropose {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                input: options.require_path("--input")?,
                output: options.output,
            }),
            "check" => Ok(Self::MorphismCheck {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                morphism_id: options.require_id("--morphism-id")?,
                output: options.output,
            }),
            "apply" => Ok(Self::MorphismApply {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                morphism_id: options.require_id("--morphism-id")?,
                base_revision_id: options
                    .base_revision_id
                    .clone()
                    .or(options.revision_id.clone())
                    .ok_or_else(|| NativeCliError::usage("--base-revision-id <id> is required"))?,
                reviewer_id: Some(options.require_id("--reviewer-id")?),
                reason: Some(options.require_string("--reason")?),
                output: options.output,
            }),
            "reject" => Ok(Self::MorphismReject {
                store: options.require_store()?,
                case_space_id: options.require_id("--case-space-id")?,
                morphism_id: options.require_id("--morphism-id")?,
                reviewer_id: options.require_id("--reviewer-id")?,
                reason: options.require_string("--reason")?,
                revision_id: options.require_id("--revision-id")?,
                output: options.output,
            }),
            _ => Err(NativeCliError::usage("unsupported native morphism command")),
        }
    }
}

fn is_history_topology(operation: &str, args: &[OsString]) -> bool {
    operation == "history"
        && args
            .first()
            .and_then(|argument| argument.to_str())
            .is_some_and(|argument| argument == "topology")
}
