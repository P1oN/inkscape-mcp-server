//! Fixed legacy and macOS bundle paths. Mutable installation state is outside code bundles.
use std::path::{Path, PathBuf};
pub const RUNTIME_APP: &str = "Inkscape MCP Runtime.app";
pub fn library_for_executable(executable: &Path) -> Result<PathBuf, String> {
    let directory = executable.parent().ok_or("runtime directory unavailable")?;
    let root = directory.parent().ok_or("runtime root unavailable")?;
    Ok(
        if directory.file_name().is_some_and(|n| n == "MacOS")
            && root.file_name().is_some_and(|n| n == "Contents")
        {
            root.join("Resources")
        } else {
            root.join("libexec/inkscape-mcp")
        },
    )
}
pub fn library(root: &Path) -> PathBuf {
    let bundled = root.join(RUNTIME_APP).join("Contents/Resources");
    if bundled.join("package.json").is_file() {
        bundled
    } else {
        root.join("libexec/inkscape-mcp")
    }
}
fn contents(library: &Path) -> Option<&Path> {
    library.parent().filter(|p| {
        p.file_name().is_some_and(|n| n == "Contents")
            && library.file_name().is_some_and(|n| n == "Resources")
    })
}
pub fn binary(library: &Path, name: &str) -> PathBuf {
    if let Some(contents) = contents(library) {
        contents.join("MacOS").join(name)
    } else {
        library
            .parent()
            .and_then(Path::parent)
            .unwrap_or(library)
            .join("bin")
            .join(name)
    }
}
pub fn asset(library: &Path, name: &str) -> PathBuf {
    if let Some(contents) = contents(library) {
        if name == "context.so" {
            return contents.join("Frameworks/context.so");
        }
        if let Some(name) = name.strip_prefix("dbus/bin/") {
            return contents.join("MacOS").join(name);
        }
        if let Some(name) = name.strip_prefix("dbus/lib/")
            && !name.starts_with("gio/")
        {
            return contents.join("Frameworks").join(name);
        }
    }
    library.join(name)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_and_bundle_assets_resolve_without_global_state_or_links() {
        for (executable, library) in [
            ("/package/bin/inkscape-mcp", "/package/libexec/inkscape-mcp"),
            (
                "/package/Inkscape MCP Runtime.app/Contents/MacOS/inkscape-mcp",
                "/package/Inkscape MCP Runtime.app/Contents/Resources",
            ),
        ] {
            let library = Path::new(library);
            assert_eq!(
                library_for_executable(Path::new(executable)).unwrap(),
                library
            );
            assert_eq!(binary(library, "inkscape-mcp"), Path::new(executable));
            assert!(
                asset(library, "context.so").ends_with(if executable.contains(".app") {
                    "Contents/Frameworks/context.so"
                } else {
                    "libexec/inkscape-mcp/context.so"
                })
            );
            assert!(
                asset(library, "dbus/bin/gdbus").ends_with(if executable.contains(".app") {
                    "Contents/MacOS/gdbus"
                } else {
                    "libexec/inkscape-mcp/dbus/bin/gdbus"
                })
            );
        }
    }
}
