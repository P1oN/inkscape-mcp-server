//! Cheap shared change baseline and bounded, cancelable polling; no document/PNG capture.
use crate::{live::Live, live_models, live_session::Session, live_socket::Error};
use rmcp::{RoleServer, service::RequestContext};
use serde_json::{Value, json};
use std::{sync::Mutex, time::Duration};

pub fn empty() -> Value {
    live_models::change(
        None,
        &json!({"revision":"","selection":"","viewport":""}),
        &[],
    )
}
pub fn detect(session: &mut Session) -> Result<Value, Error> {
    let (token, ids) = session.require_transport()?.state_token()?;
    let change = live_models::change(session.last_token.as_ref(), &token, &ids);
    session.record_change(token, change.clone());
    Ok(change)
}
pub fn protocol_message(error: &Error) -> Option<&'static str> {
    match error {
        Error::Rejected => Some("live helper rejected the request"),
        Error::Protocol(
            message @ ("protocol version mismatch"
            | "malformed success response"
            | "response missing ok discriminator"),
        ) => Some(message),
        _ => None,
    }
}
fn wait_error(error: Error) -> String {
    protocol_message(&error)
        .map(|message| format!("Error calling tool 'live_wait_for_change': {message}"))
        .unwrap_or_else(|| error.public_message().to_string())
}
fn parameters(args: &Value) -> Result<(Duration, Duration), String> {
    let number = |key, default| -> Result<f64, String> {
        let value = crate::arguments::optional_number(args, key)?.unwrap_or(default);
        if !value.is_finite() {
            return Err(format!("{key} must be finite"));
        }
        Ok(value)
    };
    let timeout = number("timeout_s", 5.)?;
    let interval = number("poll_interval_s", 0.5)?;
    if !(0. ..=60.).contains(&timeout) {
        return Err("timeout_s must be between 0 and 60".into());
    }
    if interval <= 0. || interval > 60. {
        return Err("poll_interval_s must be greater than 0 and at most 60".into());
    }
    Ok((
        Duration::from_secs_f64(timeout),
        Duration::from_secs_f64(interval.max(0.01)),
    ))
}
pub async fn wait(
    live: &Mutex<Live>,
    args: &Value,
    context: &RequestContext<RoleServer>,
) -> Result<Value, String> {
    let (timeout, interval) = parameters(args)?;
    let deadline = tokio::time::Instant::now() + timeout;
    let mut change = detect(&mut live.lock().map_err(|_| "live session lock failed")?.session)
        .map_err(wait_error)?;
    loop {
        if change["changed"] == true {
            return Ok(change);
        }
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            // Python returns the current shared token, with empty convenience ids,
            // without replacing the persisted last observation with a timeout.
            let live = live.lock().map_err(|_| "live session lock failed")?;
            let token = live.session.last_token.as_ref().unwrap_or(&change["token"]);
            let mut result = live_models::change(None, token, &[]);
            result["timed_out"] = json!(true);
            return Ok(result);
        }
        tokio::select! {
            _ = context.ct.cancelled() => return Err("live wait cancelled".into()),
            _ = tokio::time::sleep(interval.min(remaining)) => (),
        }
        change = detect(&mut live.lock().map_err(|_| "live session lock failed")?.session)
            .map_err(wait_error)?;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn polling_parameters_have_hard_budget_and_interval_floor() {
        assert_eq!(
            parameters(&json!({})).unwrap(),
            (Duration::from_secs(5), Duration::from_millis(500))
        );
        assert_eq!(
            parameters(&json!({"timeout_s":0,"poll_interval_s":0.00001})).unwrap(),
            (Duration::ZERO, Duration::from_millis(10))
        );
        for args in [
            json!({"timeout_s":-1}),
            json!({"timeout_s":61}),
            json!({"poll_interval_s":0}),
            json!({"poll_interval_s":61}),
            json!({"timeout_s":"NaN"}),
        ] {
            assert!(parameters(&args).is_err());
        }
        assert_eq!(
            empty(),
            json!({"changed":false,"selection_changed":false,"document_changed":false,"viewport_changed":false,"timed_out":false,"token":{"revision":"","selection":"","viewport":""},"selection_ids":[]})
        );
    }
}
