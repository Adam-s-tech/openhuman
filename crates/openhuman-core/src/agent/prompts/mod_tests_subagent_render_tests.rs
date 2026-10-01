use super::*;

#[test]
fn user_memory_section_returns_empty_when_no_summaries() {
    // Empty learned context → section returns empty string and is
    // skipped by the prompt builder, so the cache boundary stays
    // exactly where it was for workspaces with no tree summaries.
    let learned = LearnedContextData::default();
    let prompt_tools: Vec<PromptTool<'_>> = Vec::new();
    let ctx = PromptContext {
        workspace_dir: Path::new("/tmp"),
        model_name: "test-model",
        agent_id: "",
        tools: &prompt_tools,
        workflows: &[],
        dispatcher_instructions: "",
        learned,
        visible_tool_names: &NO_FILTER,
        tool_call_format: ToolCallFormat::PFormat,
        connected_integrations: &[],
        connected_identities_md: String::new(),
        include_profile: false,
        include_memory_md: false,
        curated_snapshot: None,
        user_identity: None,
        personality_roster: vec![],
        agents_md_global: None,
        agents_md_local: None,
    };
    let rendered = UserMemorySection.build(&ctx).unwrap();
    assert!(rendered.is_empty());
}

#[test]
fn render_subagent_system_prompt_renders_workspace_tail() {
    let workspace = std::env::temp_dir().join(format!(
        "openhuman_prompt_subagent_{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&workspace).unwrap();

    let tools: Vec<Box<dyn Tool>> = vec![Box::new(TestTool)];
    let rendered = render_subagent_system_prompt(
        &workspace,
        "test-model",
        &[0],
        &tools,
        &[],
        "You are a focused sub-agent.",
        SubagentRenderOptions::narrow(),
        ToolCallFormat::PFormat,
        &[],
    );

    assert!(rendered.contains("## Workspace"));
    assert!(rendered.contains("## Runtime"));
    // Grounding contract is appended even by the narrow (index-based)
    // sub-agent renderer — same source const, so it can never drift from
    // `GroundingSection` / the central `build()` append.
    assert!(rendered.contains("## Grounding and tool use"));
    assert!(rendered.contains("Your tools are exactly the ones you have been given for this turn"));
    assert!(rendered.contains("Preserve numeric evidence exactly"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn subagent_prompt_defaults_to_python_and_omits_protocol_without_tools() {
    let workspace = std::env::temp_dir().join(format!(
        "openhuman_prompt_default_dialect_{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&workspace).unwrap();

    let tools: Vec<Box<dyn Tool>> = vec![Box::new(TestTool)];
    let default_rendered = render_subagent_system_prompt(
        &workspace,
        "test-model",
        &[0],
        &tools,
        &[],
        "You are a focused sub-agent.",
        SubagentRenderOptions::narrow(),
        ToolCallFormat::default(),
        &[],
    );
    assert!(default_rendered.contains("def test_tool() -> str"));
    assert!(!default_rendered.contains("test_tool[]"));

    let no_tools = render_subagent_system_prompt(
        &workspace,
        "test-model",
        &[],
        &[],
        &[],
        "You are a focused sub-agent.",
        SubagentRenderOptions::narrow(),
        ToolCallFormat::Json,
        &[],
    );
    assert!(!no_tools.contains("## Tools"));
    assert!(!no_tools.contains("## Tool Use Protocol"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn subagent_render_options_invert_definition_flags() {
    // (omit_identity, omit_safety_preamble,
    //  omit_profile, omit_memory_md)
    let options = SubagentRenderOptions::from_definition_flags(true, false, false, false);
    assert!(!options.include_identity);
    assert!(options.include_safety_preamble);
    assert!(options.include_profile);
    assert!(options.include_memory_md);
    let narrow = SubagentRenderOptions::narrow();
    let default = SubagentRenderOptions::default();
    assert_eq!(narrow.include_identity, default.include_identity);
    assert_eq!(
        narrow.include_safety_preamble,
        default.include_safety_preamble
    );
    assert_eq!(narrow.include_profile, default.include_profile);
    assert_eq!(narrow.include_memory_md, default.include_memory_md);
    // Narrow default = every flag off, including both user files.
    assert!(!narrow.include_profile);
    assert!(!narrow.include_memory_md);
}

#[test]
fn render_subagent_system_prompt_honors_identity_safety_and_skills_flags() {
    let workspace =
        std::env::temp_dir().join(format!("openhuman_prompt_opts_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::write(workspace.join("SOUL.md"), "# Soul\nContext").unwrap();
    std::fs::write(workspace.join("IDENTITY.md"), "# Identity\nContext").unwrap();

    let tools: Vec<Box<dyn Tool>> = vec![Box::new(TestTool)];
    let rendered = render_subagent_system_prompt_with_format(
        &workspace,
        "reasoning-v1",
        &[0],
        &tools,
        &[],
        "You are a specialist.",
        SubagentRenderOptions {
            include_identity: true,
            include_safety_preamble: true,
            include_profile: false,
            include_memory_md: false,
        },
        ToolCallFormat::Json,
        &[],
        None,
        None,
    );

    assert!(rendered.contains("## Project Context"));
    assert!(rendered.contains("### SOUL.md"));
    assert!(rendered.contains("## Safety"));
    // Json is a prompt-driven format (the model wraps JSON tool
    // calls in `<tool_call>` tags); it does NOT use the provider's
    // native function-calling channel. So the prose tool catalogue
    // MUST still be rendered for Json, with each tool's compact
    // argument signature so the model knows what to emit.
    // Only `ToolCallFormat::Native` gets the section omitted (see
    // the `native` branch below and the `!matches!(…, Native)`
    // guard in the renderer).
    assert!(rendered.contains("### Available Tools"));
    assert!(rendered.contains("**test_tool**"));
    assert!(rendered.contains("Arguments: `object`"));

    let native = render_subagent_system_prompt_with_format(
        &workspace,
        "reasoning-v1",
        &[0],
        &tools,
        &[],
        "You are a specialist.",
        SubagentRenderOptions::narrow(),
        ToolCallFormat::Native,
        &[],
        None,
        None,
    );
    assert!(native.contains("through native tool-calling."));
    assert!(!native.contains("## Safety"));
    // Native is the only format where the prose `## Tools` section
    // is intentionally omitted — schemas travel through the
    // provider's `tools` field instead. Regression guard against
    // the ~54k-token schema duplication from the #447 PR.
    assert!(!native.contains("\n## Tools\n"));
    assert!(!native.contains("Parameters:"));

    let _ = std::fs::remove_dir_all(workspace);
}

/// Render a sub-agent prompt over a scratch workspace holding `files`, with the
/// PFormat tool-call format and the single `TestTool`.
fn render_with_files(files: &[(&str, &str)], options: SubagentRenderOptions) -> String {
    let workspace = tempfile::tempdir().expect("tempdir");
    for (name, body) in files {
        std::fs::write(workspace.path().join(name), body).unwrap();
    }
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(TestTool)];
    render_subagent_system_prompt(
        workspace.path(),
        "test-model",
        &[0],
        &tools,
        &[],
        "You are a specialist agent.",
        options,
        ToolCallFormat::PFormat,
        &[],
    )
}

fn only(identity: bool, profile: bool, memory: bool) -> SubagentRenderOptions {
    SubagentRenderOptions {
        include_identity: identity,
        include_safety_preamble: false,
        include_profile: profile,
        include_memory_md: memory,
    }
}

const PROFILE: &str = "# User Profile\nName: Jane Doe\nRole: Data scientist";
const MEMORY: &str = "# Long-term memory\nUser prefers terse Rust answers.";

#[test]
fn subagent_profile_md_follows_include_profile_flag_independent_of_identity() {
    // include_profile=true injects PROFILE.md even with the identity preamble
    // omitted (SOUL/IDENTITY stay hidden); with identity on, all three appear.
    let soul = [
        ("SOUL.md", "# Soul\nctx"),
        ("IDENTITY.md", "# Identity\nctx"),
        ("PROFILE.md", PROFILE),
    ];
    let rendered = render_with_files(&soul, only(false, true, false));
    assert!(rendered.contains("### PROFILE.md"), "{rendered}");
    assert!(rendered.contains("Jane Doe"), "{rendered}");
    assert!(!rendered.contains("## Project Context"), "{rendered}");
    assert!(
        !rendered.contains("### SOUL.md") && !rendered.contains("### IDENTITY.md"),
        "{rendered}"
    );

    let rendered = render_with_files(&soul, only(true, true, false));
    for header in [
        "## Project Context",
        "### SOUL.md",
        "### IDENTITY.md",
        "### PROFILE.md",
    ] {
        assert!(rendered.contains(header), "{header}: {rendered}");
    }

    // include_profile=false never leaks the file, even when it is on disk.
    let rendered = render_with_files(&soul, SubagentRenderOptions::narrow());
    assert!(!rendered.contains("### PROFILE.md"), "{rendered}");
    assert!(!rendered.contains("ctx"), "{rendered}");
}

#[test]
fn render_subagent_system_prompt_silently_skips_missing_profile_md() {
    // Pre-onboarding workspaces have no PROFILE.md: no orphan header, no
    // "[File not found]" placeholder.
    let rendered = render_with_files(&[], only(false, true, false));
    assert!(!rendered.contains("### PROFILE.md"), "{rendered}");
    assert!(
        !rendered.contains("[File not found: PROFILE.md]"),
        "{rendered}"
    );
}

#[test]
fn subagent_definition_flags_gate_profile_md() {
    // omit_profile=false opts IN even with omit_identity=true.
    let rendered = render_with_files(
        &[("PROFILE.md", PROFILE)],
        SubagentRenderOptions::from_definition_flags(true, true, false, false),
    );
    assert!(rendered.contains("### PROFILE.md"), "{rendered}");
    assert!(rendered.contains("Jane Doe"), "{rendered}");

    // A narrow specialist (every omit_* true) must not see it.
    let rendered = render_with_files(
        &[("PROFILE.md", PROFILE)],
        SubagentRenderOptions::from_definition_flags(true, true, true, true),
    );
    assert!(!rendered.contains("### PROFILE.md"), "{rendered}");
    assert!(!rendered.contains("Jane Doe"), "{rendered}");
}

#[test]
fn render_subagent_system_prompt_frames_memory_md_as_background() {
    // GH-4745: sub-agent MEMORY.md must share the background-memory frame so a
    // fresh thread does not read it as prior in-thread conversation.
    let rendered = render_with_files(
        &[(
            "MEMORY.md",
            "# Long-term memory\nReviewed `def f(x)` last week; user prefers terse notes.",
        )],
        only(false, false, true),
    );
    assert!(
        rendered.contains("### MEMORY.md") && rendered.contains("terse notes"),
        "{rendered}"
    );
    let frame_at = rendered
        .find("background — not this conversation")
        .unwrap_or_else(|| panic!("missing background frame: {rendered}"));
    let heading_at = rendered.find("### MEMORY.md").unwrap();
    assert!(
        frame_at < heading_at,
        "frame must precede block: {rendered}"
    );
}

#[test]
fn render_subagent_system_prompt_omits_memory_framing_when_no_memory_content() {
    // include_memory_md=true but no MEMORY.md on disk: no dangling frame.
    let rendered = render_with_files(&[], only(false, false, true));
    assert!(
        !rendered.contains("background — not this conversation"),
        "{rendered}"
    );
}

#[test]
fn subagent_memory_md_follows_include_memory_flag() {
    let rendered = render_with_files(&[("MEMORY.md", MEMORY)], only(false, false, true));
    assert!(rendered.contains("### MEMORY.md"), "{rendered}");
    assert!(rendered.contains("terse Rust answers"), "{rendered}");

    let rendered = render_with_files(&[("MEMORY.md", MEMORY)], SubagentRenderOptions::narrow());
    assert!(!rendered.contains("### MEMORY.md"), "{rendered}");
    assert!(!rendered.contains("terse Rust answers"), "{rendered}");
}

#[test]
fn render_subagent_system_prompt_code_formats_list_signatures_and_protocol() {
    let workspace =
        std::env::temp_dir().join(format!("openhuman_prompt_code_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&workspace).unwrap();

    let tools: Vec<Box<dyn Tool>> = vec![Box::new(TestTool)];
    for (format, signature, example) in [
        (
            ToolCallFormat::Python,
            "def test_tool() -> str  # tool desc",
            "read_file(path=\"src/main.rs\", limit=20)",
        ),
        (
            ToolCallFormat::TypeScript,
            "function test_tool(): string;  // tool desc",
            "read_file({path: \"src/main.rs\", limit: 20})",
        ),
    ] {
        let rendered = render_subagent_system_prompt_with_format(
            &workspace,
            "reasoning-v1",
            &[0],
            &tools,
            &[],
            "You are a specialist.",
            SubagentRenderOptions::narrow(),
            format,
            &[],
            None,
            None,
        );
        // A prompt-driven format: the child must be told which tools exist,
        // as signatures, and how to call them.
        assert!(rendered.contains("## Tools\n"), "{format:?}:\n{rendered}");
        assert!(rendered.contains(signature), "{format:?}:\n{rendered}");
        assert!(
            rendered.contains("## Tool Use Protocol"),
            "{format:?}:\n{rendered}"
        );
        assert!(rendered.contains(example), "{format:?}:\n{rendered}");
        assert!(!rendered.contains("Parameters:"), "{format:?}:\n{rendered}");
        assert!(!rendered.contains("Call as:"), "{format:?}:\n{rendered}");
        assert!(
            rendered.contains("parent agent will weave it back"),
            "{format:?}:\n{rendered}"
        );
    }

    let _ = std::fs::remove_dir_all(workspace);
}
