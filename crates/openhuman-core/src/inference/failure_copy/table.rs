//! The one table mapping a chat-failure class to the `error_type` wire token,
//! the `source` tag, the retry verdict and the user-facing copy.
//!
//! Matching a raw error string to a class is the ladder in
//! `web_chat::web_errors::classify`; the strings and wire values live only
//! here. A literal fixture (`web_chat/web_tests_error_copy_fixture_tests.rs`)
//! pins every row.

/// One row: what the frontend branches on (`error_type`, `source`,
/// `retryable`) and what the user reads (`copy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FailureCopy {
    /// Stable `error_type` wire token of the `chat_error` event.
    pub(crate) error_type: &'static str,
    /// Where the limit originated (`provider`, `openhuman_budget`,
    /// `agent_loop`, `openhuman_billing`, `transport`, `config`, `auth`).
    pub(crate) source: &'static str,
    /// Whether the same prompt can be retried in the same thread.
    pub(crate) retryable: bool,
    /// User copy. A few classes take a suffix at the call site (retry-after
    /// hint, provider detail block); the base text is always this string.
    pub(crate) copy: &'static str,
}

/// A failure class the chat surface renders copy for. Several classifier arms
/// share a class (the "temporarily unavailable" copy is reached from four).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FailureClass {
    CodexSessionExpired,
    SessionExpired,
    ActionBudget,
    MaxIterations,
    TurnTimeout,
    EmptyResponse,
    ChatTemplateRejected,
    RateLimited,
    RateLimitedBilling,
    ManagedRateLimited,
    Timeout,
    AuthError,
    BudgetExhausted,
    ManagedBudgetExhausted,
    ProviderUnavailable,
    ManagedUnavailable,
    PayloadTooLarge,
    ContextOverflow,
    ModelConfigRejected,
    ModelUnavailable,
    CapabilityUnsupported,
    MalformedHistory,
    RequestRejected,
    ManagedRequestRejected,
    ManagedMalformedRequest,
    Network,
    Inference,
    ManagedInternal,
}

/// The row for `class`.
pub(crate) fn failure_copy(class: FailureClass) -> &'static FailureCopy {
    use FailureClass::*;
    macro_rules! row {
        ($ty:expr, $src:expr, $retry:expr, $copy:expr) => {{
            static ROW: FailureCopy = FailureCopy {
                error_type: $ty,
                source: $src,
                retryable: $retry,
                copy: $copy,
            };
            &ROW
        }};
    }
    match class {
        CodexSessionExpired => row!(
            "provider_error",
            "auth",
            false,
            "Your Codex session has expired. Please reconnect it in \
                 Settings → Integrations."
        ),
        SessionExpired => row!(
            "session_expired",
            "auth",
            false,
            "Your OpenHuman session has expired. \
                 Please sign in again to continue."
        ),
        ActionBudget => row!(
            "action_budget_exceeded",
            "openhuman_budget",
            true,
            "You've hit OpenHuman's per-hour action budget — this is a local safety cap, \
                 not your AI provider. The window decays gradually; you can keep chatting in \
                 this thread and tool-heavy steps will resume as the budget refills."
        ),
        MaxIterations => row!(
            "max_iterations",
            "agent_loop",
            true,
            "The agent ran the maximum number of tool steps for one turn without \
                 finishing. This usually means a tool kept failing (often a rate limit on a \
                 web fetch). You can retry the same question in this thread once the \
                 underlying limit clears."
        ),
        TurnTimeout => row!(
            "turn_timeout",
            "agent_loop",
            true,
            "This turn ran past its time budget without finishing and was \
                 stopped so it wouldn't hang. This usually means a tool call or a \
                 delegated sub-agent stalled. You can retry your question in this thread."
        ),
        EmptyResponse => row!(
            "empty_response",
            "agent_loop",
            true,
            "The model returned an empty response. Please retry. If it keeps happening, \
                 try a different model or check its setup in Connections → API keys → LLM."
        ),
        ChatTemplateRejected => row!(
            "chat_template_rejected",
            "provider",
            true,
            "This model's chat template rejected the request — it isn't the model, the \
                 temperature, or your API key. Local models without native tool calling are \
                 driven through their own chat template, and some templates refuse the \
                 message shape of a tool step. Start a new chat to reset the history, or \
                 pick a model with native tool support in Connections → API keys → LLM."
        ),
        RateLimited => row!(
            "rate_limited",
            "provider",
            true,
            "Your AI provider is rate-limiting requests. This is a transient upstream \
                 limit, not a thread-level block — you can retry in this thread."
        ),
        RateLimitedBilling => row!(
            "rate_limited",
            "provider",
            false,
            "Your AI provider is rejecting requests for billing or plan reasons \
             (out of credits, plan limit, or unavailable model). Retrying won't \
             help — open Settings to top up, upgrade your plan, or pick a \
             different model."
        ),
        ManagedRateLimited => row!(
            "rate_limited",
            "provider",
            true,
            "Your AI provider is rate-limiting requests. You can retry in this thread."
        ),
        Timeout => row!(
            "timeout",
            "transport",
            true,
            "The request timed out. Please check your connection and try again."
        ),
        AuthError => row!(
            "auth_error",
            "config",
            false,
            "There's an authentication issue with the AI provider. Please check your API key in settings."
        ),
        // Source is `provider` instead when the 402 carries a non-OpenHuman
        // provider envelope; the classifier applies that override.
        BudgetExhausted => row!(
            "budget_exhausted",
            "openhuman_billing",
            false,
            "You're out of credits, so I can't run the managed (cloud) model right now. \
     You can top up your credits or pick a plan to continue — or, if you've enabled a \
     local model like Ollama, switch routing to \"Use Your Own Models\" in Connections → API keys → LLM."
        ),
        ManagedBudgetExhausted => row!(
            "budget_exhausted",
            "openhuman_billing",
            false,
            "You're out of credits. Top up, or switch to 'Use Your Own Models' \
                 in Settings."
        ),
        ProviderUnavailable => row!(
            "provider_error",
            "provider",
            true,
            "The AI provider is temporarily unavailable. Please try again later."
        ),
        ManagedUnavailable => row!(
            "provider_error",
            "provider",
            true,
            "The AI service is temporarily unavailable — we've been notified. \
                     Please try again shortly."
        ),
        PayloadTooLarge => row!(
            "payload_too_large",
            "config",
            false,
            "Your message or attachment is too large for this model. Shorten it \
                 or remove the attachment — or start a new thread."
        ),
        ContextOverflow => row!(
            "context_overflow",
            "config",
            false,
            "The conversation is too long. Please start a new chat."
        ),
        ModelConfigRejected => row!(
            "model_unavailable",
            "config",
            false,
            "Your AI provider rejected the request's model or temperature setting. \
                 Check your model and routing in Settings → LLM."
        ),
        ModelUnavailable => row!(
            "model_unavailable",
            "config",
            false,
            "The selected model isn't available on your provider. Check your model settings."
        ),
        CapabilityUnsupported => row!(
            "capability_unsupported",
            "config",
            false,
            "This model can't process images. Remove the attachment or switch to a \
                 vision-capable model in Connections → API keys → LLM."
        ),
        // The de-poison guard (`run_task.rs`) has already evicted the
        // offending warm session by the time this is shown, so "send it
        // again" is literally true.
        MalformedHistory => row!(
            "provider_request_rejected",
            "provider",
            true,
            "We hit a temporary glitch in this conversation — we've cleared it. \
     Please send your message again."
        ),
        RequestRejected => row!(
            "provider_request_rejected",
            "provider",
            false,
            "The AI provider rejected the request — this is usually a model or \
                 parameter incompatibility. Try a different model in Connections → API keys → LLM."
        ),
        ManagedRequestRejected => row!(
            "provider_request_rejected",
            "provider",
            false,
            "The request was rejected — usually a model or parameter \
                         mismatch. Try a different model in Connections → API keys → LLM."
        ),
        ManagedMalformedRequest => row!(
            "provider_request_rejected",
            "provider",
            false,
            "Something went wrong with this message. Try rephrasing it — \
                         or start a new thread if it keeps happening."
        ),
        Network => row!(
            "network",
            "transport",
            true,
            "The connection to the AI service dropped mid-response — usually a \
                 sleep/wake or network change. Please try again."
        ),
        Inference => row!(
            "inference",
            "provider",
            true,
            "Something went wrong. Please try again."
        ),
        ManagedInternal => row!(
            "inference",
            "provider",
            true,
            "Something went wrong — we've been notified. Please try again."
        ),
    }
}
