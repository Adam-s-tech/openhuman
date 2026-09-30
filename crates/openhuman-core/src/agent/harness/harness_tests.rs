use super::parse_tests::tools_to_openai_format;
use crate::tools;
use std::sync::Arc;

fn build_tool_instructions(tools: &[Box<dyn tinytools::Tool>]) -> String {
    let specs = tools.iter().map(|tool| tool.spec()).collect::<Vec<_>>();
    tinytools_agent::dialect::XmlDialect::instructions(&specs)
}

#[path = "harness_tool_call_parsing_tests.rs"]
mod harness_tool_call_parsing_tests;
