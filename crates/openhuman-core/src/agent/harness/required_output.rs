//! Required structured-output validation & repair (issue #4117); the pure
//! primitives live in [`tinyagents_harness::config`].

pub(crate) use tinyagents_harness::config::{
    output_satisfies_contract, repair_instruction, synthesize_block,
};
