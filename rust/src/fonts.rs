//! Fontconfig queries use fixed executables and argument lists, never a shell.
use crate::process;
use regex::Regex;
use std::{collections::HashSet, sync::LazyLock};

static LAYOUT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\p{Cc}\p{Cf}\p{Zl}\p{Zp}\p{Zs}]").unwrap());

pub fn generic(family: &str) -> bool {
    matches!(
        family.trim().to_lowercase().as_str(),
        "serif"
            | "sans-serif"
            | "monospace"
            | "cursive"
            | "fantasy"
            | "system-ui"
            | "ui-serif"
            | "ui-sans-serif"
            | "ui-monospace"
            | "ui-rounded"
            | "math"
            | "emoji"
            | "fangsong"
    )
}

fn query(name: &str, args: &[String], cap: usize) -> Option<String> {
    let binary = process::binary(name)?;
    let cap = cap.min(32 * 1024 * 1024);
    let result = process::run_bounded(&binary, args, process::timeout(), cap).ok()?;
    if !result.success || result.timed_out || result.stdout.len() >= cap {
        return None;
    }
    String::from_utf8(result.stdout).ok()
}

pub fn installed(cap: usize) -> Option<HashSet<String>> {
    Some(
        query("fc-list", &[":".into(), "family".into()], cap)?
            .lines()
            .flat_map(|line| line.split(','))
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

fn significant(text: &str) -> Vec<char> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for c in text.chars() {
        if !LAYOUT.is_match(&c.to_string()) && seen.insert(c) {
            result.push(c);
        }
    }
    result
}

pub fn uncovered(family: &str, text: &str, cap: usize) -> Option<String> {
    let points = significant(text);
    if points.is_empty() {
        return Some(String::new());
    }
    if family.trim().is_empty() || generic(family) {
        return None;
    }
    let matched = query(
        "fc-match",
        &[
            "-f".into(),
            "%{family}|%{file}".into(),
            family.trim().into(),
        ],
        cap,
    )?;
    let (aliases, _) = matched.trim().split_once('|')?;
    if !aliases
        .split(',')
        .any(|a| a.trim().eq_ignore_ascii_case(family.trim()))
    {
        return None;
    }
    let charset = query(
        "fc-list",
        &[format!(":family={}", family.trim()), "charset".into()],
        cap,
    )?;
    let mut ranges = Vec::new();
    for line in charset.lines() {
        for token in line
            .trim()
            .strip_prefix(":charset=")
            .unwrap_or(line.trim())
            .split_whitespace()
        {
            let (a, b) = token.split_once('-').unwrap_or((token, token));
            if let (Ok(a), Ok(b)) = (u32::from_str_radix(a, 16), u32::from_str_radix(b, 16)) {
                ranges.push((a.min(b), a.max(b)));
            }
        }
    }
    if ranges.is_empty() {
        return None;
    }
    Some(
        points
            .into_iter()
            .filter(|c| !ranges.iter().any(|(a, b)| (*a..=*b).contains(&(*c as u32))))
            .collect(),
    )
}

pub fn suggest(chars: &str, cap: usize) -> Option<String> {
    let points = significant(chars);
    if points.is_empty() {
        return None;
    }
    let argument = format!(
        ":charset={}",
        points
            .iter()
            .map(|c| format!("{:04x}", *c as u32))
            .collect::<Vec<_>>()
            .join(" ")
    );
    query("fc-list", &[argument, "family".into()], cap)?
        .lines()
        .filter_map(|line| line.split(',').next())
        .map(str::trim)
        .find(|s| !s.is_empty() && !generic(s))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_is_not_missing_glyphs() {
        assert_eq!(
            significant("a a\t\n\u{200b}\u{feff}ああ\u{a0}"),
            vec!['a', 'あ']
        );
        assert!(generic(" UI-Monospace "));
    }
}
