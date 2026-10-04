use std::{env, fs, os::unix::net::UnixListener, path::PathBuf, thread, time::Duration};
fn main() {
    let exe = env::current_exe().unwrap();
    let mode = fs::read_to_string(exe.with_extension("mode")).unwrap_or_default();
    if env::args().any(|a| a == "--user-data-directory") {
        println!(
            "{}",
            exe.parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("profile")
                .display()
        );
        return;
    }
    if mode == "fail" {
        std::process::exit(2);
    }
    if let Some(address) =
        env::args().find_map(|a| a.strip_prefix("--address=unix:path=").map(str::to_owned))
    {
        let socket = PathBuf::from(address);
        fs::write(socket.with_extension("pid"), std::process::id().to_string()).unwrap();
        let _listener = if mode == "timeout" {
            None
        } else {
            Some(UnixListener::bind(&socket).unwrap())
        };
        loop {
            thread::sleep(Duration::from_millis(10));
        }
    }
    let session = PathBuf::from(env::var_os("INKSCAPE_MCP_MANAGED_DIR").unwrap());
    fs::write(session.join("gui-started"), std::process::id().to_string()).unwrap();
    if mode == "exit" {
        return;
    }
    while !session.join("finish-gui").exists() {
        thread::sleep(Duration::from_millis(10));
    }
}
