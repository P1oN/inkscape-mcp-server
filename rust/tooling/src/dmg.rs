//! Fixed, headless Finder presentation. No Finder automation or user settings changes.
use crate::common::*;
use std::{fs, path::Path, process::Command};

pub const ICON: &[u8] = include_bytes!("../../../runtime/manager/artwork/Manager.icns");
pub const BACKGROUND: &[u8] =
    include_bytes!("../../../runtime/manager/artwork/installer-background.tiff");
const LAYOUT: &str = include_str!("../../../runtime/manager/dmg-layout.m");

fn mount(image: &Path, point: &Path, readonly: bool) -> Result<()> {
    let mut command = Command::new("/usr/bin/hdiutil");
    command.args(["attach", "-nobrowse", "-noautoopen"]);
    if readonly {
        command.arg("-readonly");
    }
    output(command.arg("-mountpoint").arg(point).arg(image))?;
    Ok(())
}

fn detach(point: &Path) -> Result<()> {
    output(Command::new("/usr/bin/hdiutil").arg("detach").arg(point))?;
    Ok(())
}

pub fn build(root: &Path, destination: &Path) -> Result<()> {
    ensure(cfg!(target_os = "macos"), "DMG creation requires macOS")?;
    ensure(
        !destination.exists() && !destination.is_symlink(),
        "DMG output already exists; preserved",
    )?;
    ensure(
        root.join("Inkscape MCP Manager.app/Contents/Resources/installer-background.tiff")
            .is_file(),
        "Manager must contain the installer background before app signing",
    )?;
    let scratch = tempfile::tempdir()?;
    let scratch_path = scratch.path().canonicalize()?;
    let source = scratch_path.join("dmg-layout.m");
    let helper = scratch_path.join("dmg-layout");
    fs::write(&source, LAYOUT)?;
    output(
        Command::new("/usr/bin/clang")
            .args([
                "-fobjc-arc",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-Wno-deprecated-declarations",
                "-framework",
                "Foundation",
                "-framework",
                "CoreServices",
            ])
            .arg(&source)
            .arg("-o")
            .arg(&helper),
    )?;
    let writable = scratch_path.join("layout.dmg");
    let point = scratch_path.join("mount");
    fs::create_dir(&point)?;
    output(
        Command::new("/usr/bin/hdiutil")
            .args([
                "create",
                "-format",
                "UDRW",
                "-fs",
                "HFS+",
                "-volname",
                "Inkscape MCP Manager",
            ])
            .arg("-srcfolder")
            .arg(root)
            .arg(&writable),
    )?;
    // Do not recursively delete a scratch directory if a mounted volume cannot detach.
    // Even a failed/timed-out attach can have mounted it: attempt cleanup on that path too.
    let result: Result<()> = (|| {
        mount(&writable, &point, false)?;
        // Only mutate the private image, preserving the caller's root for retries.
        #[cfg(unix)]
        std::os::unix::fs::symlink("/Applications", point.join("Applications"))?;
        output(Command::new(&helper).arg("write").arg(&point))?;
        output(Command::new(&helper).arg("verify").arg(&point))?;
        Ok(())
    })();
    if let Err(error) = detach(&point) {
        let retained = scratch.keep();
        return Err(format!(
            "DMG detach failed; scratch preserved at {}: {error}; preparation: {result:?}",
            retained.display()
        )
        .into());
    }
    result?;
    output(
        Command::new("/usr/bin/hdiutil")
            .args(["convert", "-format", "UDZO"])
            .arg(&writable)
            .arg("-o")
            .arg(destination),
    )?;
    // Check the compressed image from a different mount path. Alias resolution must
    // stay on this volume rather than depend on the temporary build mount/host path.
    let final_point = scratch_path.join("verify");
    fs::create_dir(&final_point)?;
    let result: Result<()> = (|| {
        mount(destination, &final_point, true)?;
        output(Command::new(&helper).arg("verify").arg(&final_point))?;
        Ok(())
    })();
    if let Err(error) = detach(&final_point) {
        let retained = scratch.keep();
        return Err(format!(
            "Final DMG detach failed; scratch preserved at {}: {error}; verification: {result:?}",
            retained.display()
        )
        .into());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    #[ignore = "native hdiutil mount/convert/remount; run explicitly"]
    fn compressed_image_resolves_background_after_remount() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        fs::create_dir(&root).unwrap();
        let resources = root.join("Inkscape MCP Manager.app/Contents/Resources");
        fs::create_dir_all(&resources).unwrap();
        fs::write(resources.join("installer-background.tiff"), BACKGROUND).unwrap();
        // Conversion fails after layout creation when its parent does not exist.
        // Retrying the same input must not encounter a leftover Applications link.
        let failed_image = temp.path().join("missing/Manager.dmg");
        assert!(build(&root, &failed_image).is_err());
        assert!(!failed_image.exists());
        assert!(!root.join("Applications").is_symlink());
        let image = temp.path().join("Manager.dmg");
        build(&root, &image).unwrap();
        assert!(!root.join("Applications").is_symlink());
        assert_eq!(
            fs::read(resources.join("installer-background.tiff")).unwrap(),
            BACKGROUND
        );
        assert!(image.metadata().unwrap().len() > 1024);
        assert!(build(&root, &image).is_err());
    }
}
