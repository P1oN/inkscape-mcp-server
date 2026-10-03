//! Optional telemetry. Never attach MCP arguments, results or document contents.

pub fn init() -> Option<sentry::ClientInitGuard> {
    let dsn = std::env::var("SENTRY_DSN").ok()?;
    if dsn.trim().is_empty() {
        return None;
    }
    let dsn = match dsn.parse::<sentry::types::Dsn>() {
        Ok(dsn) => dsn,
        Err(_) => {
            eprintln!("Invalid SENTRY_DSN; telemetry disabled");
            return None;
        }
    };
    let sample_rate = std::env::var("SENTRY_TRACES_SAMPLE_RATE")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
        .unwrap_or(0.1);
    Some(sentry::init(
        sentry::ClientOptions::new()
            .dsn(&dsn.to_string())
            .release(
                std::env::var("SENTRY_RELEASE")
                    .unwrap_or_else(|_| format!("inkscape-mcp-rust@{}", env!("CARGO_PKG_VERSION"))),
            )
            .environment(
                std::env::var("SENTRY_ENVIRONMENT").unwrap_or_else(|_| "development".into()),
            )
            .traces_sample_rate(sample_rate)
            .server_name("inkscape-mcp")
            .send_default_pii(false)
            .before_send(|event| Some(scrub_error(event))),
    ))
}

pub struct ToolTiming(sentry::Transaction);

impl Drop for ToolTiming {
    fn drop(&mut self) {
        self.0.clone().finish();
    }
}

pub fn tool_timing(name: &str) -> ToolTiming {
    ToolTiming(sentry::start_transaction(sentry::TransactionContext::new(
        name, "mcp.tool",
    )))
}

/// Fixed categories only: never classify by or send dynamic error text.
#[derive(Clone, Copy)]
pub enum Failure {
    ProcessStart,
    ProcessTimeout,
    ProcessCrash,
    LiveUncertain,
    Internal,
}
impl Failure {
    fn index(self) -> usize {
        self as usize
    }
    fn code(self) -> &'static str {
        match self {
            Self::ProcessStart => "process_start",
            Self::ProcessTimeout => "process_timeout",
            Self::ProcessCrash => "process_crash",
            Self::LiveUncertain => "live_uncertain",
            Self::Internal => "internal",
        }
    }
}

// Fixed storage and one event per category/minute; repeated failures cannot flood
// the free quota. No disk queue, paths, arguments, stdout or stderr are retained.
struct Budget([Option<std::time::Instant>; 5]);
impl Budget {
    fn admit(&mut self, kind: Failure, now: std::time::Instant) -> bool {
        let slot = &mut self.0[kind.index()];
        if slot.is_some_and(|last| now.duration_since(last) < std::time::Duration::from_secs(60)) {
            return false;
        }
        *slot = Some(now);
        true
    }
}
pub fn failure(kind: Failure) {
    if !sentry::Hub::current()
        .client()
        .is_some_and(|client| client.is_enabled())
    {
        return;
    }
    static BUDGET: std::sync::Mutex<Budget> = std::sync::Mutex::new(Budget([None; 5]));
    // Poisoned/contended telemetry must never hold up the actual operation.
    if !BUDGET
        .try_lock()
        .is_ok_and(|mut budget| budget.admit(kind, std::time::Instant::now()))
    {
        return;
    }
    let mut event = sentry::protocol::Event {
        level: sentry::Level::Error,
        fingerprint: vec!["inkscape-mcp".into(), kind.code().into()].into(),
        ..Default::default()
    };
    event.tags.insert("failure_kind".into(), kind.code().into());
    event.exception.values.push(sentry::protocol::Exception {
        ty: "McpFailure".into(),
        value: Some("MCP subsystem failure (details kept locally)".into()),
        stacktrace: sentry::integrations::backtrace::current_stacktrace(),
        ..Default::default()
    });
    sentry::capture_event(event);
}
pub fn internal_error(message: &'static str, data: Option<serde_json::Value>) -> rmcp::ErrorData {
    failure(Failure::Internal);
    rmcp::ErrorData::internal_error(message, data)
}

fn clean_stack(stack: &mut sentry::protocol::Stacktrace) {
    for frame in &mut stack.frames {
        frame.abs_path = None;
        frame.package = None;
        frame.vars.clear();
        frame.pre_context.clear();
        frame.post_context.clear();
        frame.context_line = None;
        if let Some(filename) = &frame.filename {
            // Handle Unix and Windows separators without preserving prefixes.
            frame.filename = filename.rsplit(['/', '\\']).next().map(str::to_owned);
        }
    }
}
fn scrub_error(event: sentry::protocol::Event<'static>) -> sentry::protocol::Event<'static> {
    // Reconstruct an allowlisted payload so future SDK integrations cannot
    // accidentally send request/context/thread/local data or log messages.
    let code = event
        .tags
        .get("failure_kind")
        .filter(|code| {
            [
                "process_start",
                "process_timeout",
                "process_crash",
                "live_uncertain",
                "internal",
            ]
            .contains(&code.as_str())
        })
        .cloned();
    let mut cleaned = sentry::protocol::Event {
        event_id: event.event_id,
        timestamp: event.timestamp,
        level: event.level,
        platform: event.platform,
        release: event.release,
        environment: event.environment,
        sdk: event.sdk,
        message: Some("MCP server failure".into()),
        ..Default::default()
    };
    if let Some(code) = code {
        cleaned.fingerprint = vec!["inkscape-mcp".into(), code.clone().into()].into();
        cleaned.tags.insert("failure_kind".into(), code);
    }
    cleaned
        .tags
        .insert("os".into(), std::env::consts::OS.into());
    cleaned
        .tags
        .insert("arch".into(), std::env::consts::ARCH.into());
    cleaned.exception = event.exception;
    for exception in &mut cleaned.exception.values {
        exception.ty = "McpFailure".into();
        exception.value = Some("MCP server failure (details kept locally)".into());
        exception.mechanism = None;
        exception.module = None;
        exception.thread_id = None;
        if let Some(stack) = &mut exception.raw_stacktrace {
            clean_stack(stack);
        }
        if let Some(stack) = &mut exception.stacktrace {
            clean_stack(stack);
        }
    }
    cleaned.stacktrace = event.stacktrace;
    if let Some(stack) = &mut cleaned.stacktrace {
        clean_stack(stack);
    }
    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_budget_limits_repeats_without_suppressing_other_categories() {
        let mut budget = Budget([None; 5]);
        let now = std::time::Instant::now();
        assert!(budget.admit(Failure::ProcessCrash, now));
        assert!(!budget.admit(Failure::ProcessCrash, now));
        assert!(budget.admit(Failure::LiveUncertain, now));
        assert!(!budget.admit(
            Failure::ProcessCrash,
            now + std::time::Duration::from_secs(59)
        ));
        assert!(budget.admit(
            Failure::ProcessCrash,
            now + std::time::Duration::from_secs(60)
        ));
    }

    #[test]
    fn allowlist_removes_sdk_contexts_tags_threads_and_stack_locals() {
        let mut event = sentry::protocol::Event {
            transaction: Some("private-tool-argument".into()),
            logentry: Some(sentry::protocol::LogEntry {
                message: "private-log".into(),
                params: vec![],
            }),
            ..Default::default()
        };
        event.tags.insert("filename".into(), "private.svg".into());
        event
            .tags
            .insert("failure_kind".into(), "process_crash".into());
        event.contexts.insert(
            "private-context".into(),
            sentry::protocol::Context::Other(Default::default()),
        );
        let frame = sentry::protocol::Frame {
            filename: Some("C:\\private\\main.rs".into()),
            package: Some("/private/bin".into()),
            vars: [("private".into(), "artwork".into())].into(),
            pre_context: vec!["private-source".into()],
            post_context: vec!["private-source".into()],
            context_line: Some("private-source".into()),
            function: Some("inkscape_mcp_rust::process::run_bounded_with_env".into()),
            lineno: Some(99),
            ..Default::default()
        };
        event.stacktrace = Some(sentry::protocol::Stacktrace {
            frames: vec![frame.clone()],
            ..Default::default()
        });
        event.threads.values.push(sentry::protocol::Thread {
            name: Some("private-thread".into()),
            stacktrace: event.stacktrace.clone(),
            ..Default::default()
        });
        let clean = scrub_error(event);
        let wire = serde_json::to_string(&clean).unwrap();
        assert!(!wire.contains("private"), "{wire}");
        assert!(wire.contains("main.rs"));
        assert!(wire.contains("process_crash"));
        assert_eq!(clean.stacktrace.unwrap().frames[0].lineno, Some(99));
    }

    #[test]
    fn panic_data_is_scrubbed_but_source_locations_survive() {
        let mut event = sentry::protocol::Event {
            message: Some("private SVG content".into()),
            server_name: Some("private-host".into()),
            ..Default::default()
        };
        event
            .extra
            .insert("document".into(), "private artwork".into());
        event.exception.values.push(sentry::protocol::Exception {
            ty: "panic".into(),
            value: Some("private path".into()),
            stacktrace: Some(sentry::protocol::Stacktrace {
                frames: vec![sentry::protocol::Frame {
                    function: Some("inkscape_mcp_rust::run_server".into()),
                    filename: Some("/private/workspace/main.rs".into()),
                    abs_path: Some("/private/workspace/main.rs".into()),
                    lineno: Some(42),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        });
        let cleaned = scrub_error(event);
        let wire = serde_json::to_string(&cleaned).unwrap();
        assert!(!wire.contains("private"));
        let frame = &cleaned.exception.values[0]
            .stacktrace
            .as_ref()
            .unwrap()
            .frames[0];
        assert_eq!(frame.filename.as_deref(), Some("main.rs"));
        assert_eq!(frame.lineno, Some(42));
        assert!(frame.abs_path.is_none());
    }
}
