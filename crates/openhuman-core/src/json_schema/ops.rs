//! Business logic for [`super`] lives in `tinyagents-harness`
//! (`tool::schema_walk`); this module only re-exports it under the names the
//! host's callers already use. See [`super`] for why this domain is neutral.

pub(crate) use tinyagents_harness::tool::{
    compute_primary_array_path, compute_primary_array_path_from_value, missing_required_args,
    response_fields_from_schema, unsupported_arg_names,
};
