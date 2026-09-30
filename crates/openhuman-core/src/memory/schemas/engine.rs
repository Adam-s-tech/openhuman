//! Schemas and handlers for the memory-engine RPCs: `engines_list`,
//! `engine_get`, `engine_set`, `engine_migrate`, `engine_migrate_status`.
//!
//! Registered with the provider family, which is never capability-gated:
//! switching engines must stay possible whatever the current engine advertises.

use serde_json::{Map, Value};

use crate::core::all::{ControllerFuture, RegisteredController};
use crate::core::{ControllerSchema, FieldSchema, TypeSchema};
use crate::memory::rpc::{
    self, EngineTargetParams, MigrateCancelParams, MigrateParams, MigrateStatusParams,
};

use super::{parse_params, to_json};

pub(super) const FUNCTIONS: &[&str] = &[
    "engines_list",
    "engine_get",
    "engine_set",
    "engine_migrate",
    "engine_migrate_status",
    "engine_migrate_cancel",
];

pub(super) fn controllers() -> Vec<RegisteredController> {
    vec![
        RegisteredController {
            schema: schema("engines_list").unwrap(),
            handler: handle_engines_list,
        },
        RegisteredController {
            schema: schema("engine_get").unwrap(),
            handler: handle_engine_get,
        },
        RegisteredController {
            schema: schema("engine_set").unwrap(),
            handler: handle_engine_set,
        },
        RegisteredController {
            schema: schema("engine_migrate").unwrap(),
            handler: handle_engine_migrate,
        },
        RegisteredController {
            schema: schema("engine_migrate_status").unwrap(),
            handler: handle_engine_migrate_status,
        },
        RegisteredController {
            schema: schema("engine_migrate_cancel").unwrap(),
            handler: handle_engine_migrate_cancel,
        },
    ]
}

fn field(name: &'static str, ty: TypeSchema, comment: &'static str, required: bool) -> FieldSchema {
    FieldSchema {
        name,
        ty,
        comment,
        required,
    }
}

fn opt_string() -> TypeSchema {
    TypeSchema::Option(Box::new(TypeSchema::String))
}

fn target_inputs() -> Vec<FieldSchema> {
    vec![
        field(
            "driver",
            TypeSchema::String,
            "Engine id from engines_list (tinymemory, tinyhumans, supermemory, mem0, cognee, cortex, agentmemory).",
            true,
        ),
        field(
            "endpoint",
            opt_string(),
            "Base URL for engines that take one. Ignored for tinyhumans.",
            false,
        ),
        field(
            "deployment",
            opt_string(),
            "cloud | self_hosted, for engines with more than one deployment.",
            false,
        ),
        field(
            "api_key",
            opt_string(),
            "API key. Stored in the OS keychain, never in config, and never returned.",
            false,
        ),
    ]
}

fn state_outputs() -> Vec<FieldSchema> {
    vec![
        field(
            "driver",
            TypeSchema::String,
            "Bound engine id (null after a fallback).",
            true,
        ),
        field(
            "endpoint",
            opt_string(),
            "Endpoint in use, when the engine has one.",
            false,
        ),
        field(
            "deployment",
            opt_string(),
            "Deployment in use, when the engine has several.",
            false,
        ),
        field(
            "has_credential",
            TypeSchema::Bool,
            "Whether a credential is available. The credential itself is never returned.",
            true,
        ),
        field(
            "class",
            TypeSchema::String,
            "How the engine is bound: module | external | null.",
            true,
        ),
        field(
            "fell_back_from",
            opt_string(),
            "The engine that was asked for and refused, when this is a fallback.",
            false,
        ),
        field(
            "last_error",
            opt_string(),
            "Why the fallback happened; null when clean.",
            false,
        ),
    ]
}

pub(super) fn schema(function: &str) -> Option<ControllerSchema> {
    Some(match function {
        "engines_list" => ControllerSchema {
            namespace: "memory",
            function: "engines_list",
            description: "The memory engines this build offers, and which one is active.",
            inputs: vec![],
            outputs: vec![
                field("engines", TypeSchema::Array(Box::new(TypeSchema::Json)), "Engine descriptors: id, label, description, needs_endpoint, needs_key, key_optional, deployments, default_endpoint, hosted, capabilities.", true),
                field("active", TypeSchema::String, "Id of the bound engine.", true),
            ],
        },
        "engine_get" => ControllerSchema {
            namespace: "memory",
            function: "engine_get",
            description: "The active memory engine's configuration and binding state. Never returns a secret.",
            inputs: vec![],
            outputs: state_outputs(),
        },
        "engine_set" => ControllerSchema {
            namespace: "memory",
            function: "engine_set",
            description: "Switch the memory engine. Stores the key in the keychain, writes [subsystems.memory], and rebinds without a restart. Existing memories are not copied; use engine_migrate for that.",
            inputs: target_inputs(),
            outputs: state_outputs(),
        },
        "engine_migrate" => ControllerSchema {
            namespace: "memory",
            function: "engine_migrate",
            description: "Copy every memory from the active engine into another, then switch to it. Returns a job id; poll engine_migrate_status.",
            inputs: vec![field(
                "to",
                TypeSchema::Json,
                "The target engine: { driver, endpoint?, deployment?, api_key? }.",
                true,
            )],
            outputs: vec![field("job_id", TypeSchema::String, "Migration job id.", true)],
        },
        "engine_migrate_status" => ControllerSchema {
            namespace: "memory",
            function: "engine_migrate_status",
            description: "Progress of an engine migration job.",
            inputs: vec![field("job_id", TypeSchema::String, "Id returned by engine_migrate.", true)],
            outputs: vec![
                field("state", TypeSchema::String, "running | done | failed.", true),
                field("copied", TypeSchema::U64, "Records copied so far.", true),
                field("total", TypeSchema::Option(Box::new(TypeSchema::U64)), "Total records when known; null otherwise.", false),
                field("error", opt_string(), "Failure reason; null unless failed.", false),
            ],
        },
        "engine_migrate_cancel" => ControllerSchema {
            namespace: "memory",
            function: "engine_migrate_cancel",
            description: "Cancel a running engine migration. The copy stops between pages and the active engine is left unchanged.",
            inputs: vec![field("job_id", TypeSchema::String, "Id returned by engine_migrate.", true)],
            outputs: vec![field("cancelled", TypeSchema::Bool, "Whether a running job was signalled; false when it had already finished.", true)],
        },
        _ => return None,
    })
}

fn handle_engines_list(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move { to_json(rpc::memory_engines_list().await?) })
}

fn handle_engine_get(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move { to_json(rpc::memory_engine_get().await?) })
}

fn handle_engine_set(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let payload = parse_params::<EngineTargetParams>(params)?;
        to_json(rpc::memory_engine_set(payload).await?)
    })
}

fn handle_engine_migrate(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let payload = parse_params::<MigrateParams>(params)?;
        to_json(rpc::memory_engine_migrate(payload).await?)
    })
}

fn handle_engine_migrate_status(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let payload = parse_params::<MigrateStatusParams>(params)?;
        to_json(rpc::memory_engine_migrate_status(payload).await?)
    })
}

fn handle_engine_migrate_cancel(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let payload = parse_params::<MigrateCancelParams>(params)?;
        to_json(rpc::memory_engine_migrate_cancel(payload).await?)
    })
}
