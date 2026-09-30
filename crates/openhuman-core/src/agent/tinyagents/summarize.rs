//! LLM-backed conversation summarization for the tinyagents harness path
//! (issue #4249). The implementation lives in
//! [`tinyagents_harness::summarization`]; the OpenHuman defaults (compact at 90%
//! of the model's window, keep the last 8 messages) are that module's defaults.

pub(super) use tinyagents_harness::summarization::{
    summarization_policy, FaultTolerantCachingSummarizer, ModelSummarizer,
};
