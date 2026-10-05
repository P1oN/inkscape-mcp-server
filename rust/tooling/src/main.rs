mod acceptance;
mod archive;
mod bootstrap;
mod clients;
mod common;
mod install;
mod invariants;
mod launcher;
mod native;
mod notices;
mod package;
mod renderer;
mod responsiveness;
mod security;
mod sentry_setup;
mod socket_acceptance;
mod startup;
mod wire;
use common::{Args, Result};
fn run() -> Result<()> {
    let args = Args::parse()?;
    match args.command.as_str() {
        "install-acceptance" => install::run(&args),
        "native-gui" | "native-inx" | "native-socket" => native::run(&args),
        "responsiveness-acceptance" => responsiveness::run(&args),
        "diagnostic-acceptance"
        | "defects-acceptance"
        | "special-file-acceptance"
        | "compare-acceptance"
        | "engine-routes-acceptance" => invariants::run(&args),
        "renderer-acceptance" => renderer::run(&args),
        "client-acceptance" => clients::run(&args),
        "sentry-setup-acceptance" => sentry_setup::run(&args),
        "notices-acceptance" => notices::acceptance(&args),
        "startup-acceptance" => startup::run(&args),
        "security-acceptance" => security::run(&args),
        "frame-acceptance" => security::frames(&args),
        "launcher-acceptance" => launcher::run(&args),
        "socket-acceptance" => socket_acceptance::run(&args),
        "package-acceptance" => acceptance::run(&args),
        "doctor-acceptance" => acceptance::doctor(&args),
        "bootstrap-native" => bootstrap::run(&args),
        "build-package" => package::run(&args),
        "source-archive" => archive::source(&args),
        "discovery" => wire::discovery(&args),
        "manifests" => wire::manifests(&args),
        "help" => {
            println!(
                "inkscape-mcp-tools: bootstrap-native | build-package | source-archive | discovery | manifests\nAcceptance: package, doctor, notices, launcher, sentry-setup, client, install, socket, security, frame, startup, responsiveness, defects, diagnostic, compare, special-file, renderer, engine-routes (append -acceptance).\nExplicit native GUI phases: native-gui | native-inx | native-socket.\nSee CONTRIBUTING.md for arguments and ownership requirements."
            );
            Ok(())
        }
        _ => Err(format!("unknown development command: {}", args.command).into()),
    }
}
fn main() {
    if std::env::var_os("INKSCAPE_MCP_TOOLING_FIXTURE").as_deref()
        == Some(std::ffi::OsStr::new("config"))
    {
        sentry_setup::fixture();
        return;
    }
    if std::env::var_os("INKSCAPE_MCP_TOOLING_FIXTURE").as_deref()
        == Some(std::ffi::OsStr::new("cancel-engine"))
    {
        responsiveness::engine_fixture();
        return;
    }
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
