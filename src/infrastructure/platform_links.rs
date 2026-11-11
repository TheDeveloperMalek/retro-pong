

#[cfg(target_arch = "wasm32")]
mod platform {
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen(inline_js = r#"
        export function retroPongOpenExternal(url) {
            window.open(url, "_blank", "noopener,noreferrer");
        }

        export function retroPongOpenAndCopy(
            url,
            successMessage,
            copyFailureMessage,
            copiedOnlyMessage,
            totalFailureMessage
        ) {
            const openedWindow = window.open(url, "_blank");
            const opened = openedWindow !== null;

            if (openedWindow) {
                openedWindow.opener = null;
            }

            if (!navigator.clipboard || !window.isSecureContext) {
                window.alert(opened ? copyFailureMessage : totalFailureMessage);
                return;
            }

            navigator.clipboard.writeText(url)
                .then(() => window.alert(opened ? successMessage : copiedOnlyMessage))
                .catch(() => window.alert(opened ? copyFailureMessage : totalFailureMessage));
        }
    "#)]
    unsafe extern "C" {
        fn retroPongOpenExternal(url: &str);
        fn retroPongOpenAndCopy(
            url: &str,
            success_message: &str,
            copy_failure_message: &str,
            copied_only_message: &str,
            total_failure_message: &str,
        );
    }

    pub fn open_url(url: &str) -> bool {
        retroPongOpenExternal(url);
        true
    }

    pub fn open_and_copy(
        url: &str,
        success_message: &str,
        copy_failure_message: &str,
        copied_only_message: &str,
        total_failure_message: &str,
    ) {
        retroPongOpenAndCopy(
            url,
            success_message,
            copy_failure_message,
            copied_only_message,
            total_failure_message,
        );
    }

}

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };

    pub fn open_url(url: &str) -> bool {
        #[cfg(target_os = "windows")]
        let result = Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();

        #[cfg(target_os = "macos")]
        let result = Command::new("open").arg(url).spawn();

        #[cfg(all(unix, not(target_os = "macos")))]
        let result = Command::new("xdg-open").arg(url).spawn();

        match result {
            Ok(_) => true,
            Err(error) => {
                bevy::log::warn!("Could not open external URL {url}: {error}");
                false
            }
        }
    }

    fn read_from_command(program: &str, args: &[&str]) -> Option<String> {
        let output = Command::new(program).args(args).output().ok()?;
        if !output.status.success() {
            return None;
        }

        let value = String::from_utf8_lossy(&output.stdout)
            .trim_end_matches(&['\r', '\n'][..])
            .to_string();

        (!value.is_empty()).then_some(value)
    }

    fn write_to_command(program: &str, args: &[&str], text: &str) -> bool {
        let Ok(mut child) = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            return false;
        };

        let Some(mut stdin) = child.stdin.take() else {
            return false;
        };

        if stdin.write_all(text.as_bytes()).is_err() {
            return false;
        }
        drop(stdin);

        child
            .wait()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    fn spawn_clipboard_command(program: &str, args: &[&str], text: &str) -> bool {
        let Ok(mut child) = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            return false;
        };

        let Some(mut stdin) = child.stdin.take() else {
            return false;
        };

        if stdin.write_all(text.as_bytes()).is_err() {
            return false;
        }
        drop(stdin);

        match child.try_wait() {
            Ok(Some(status)) => status.success(),
            Ok(None) => true,
            Err(_) => false,
        }
    }

    pub fn copy_to_clipboard(text: &str) -> bool {
        #[cfg(target_os = "windows")]
        {
            return write_to_command("clip", &[], text);
        }

        #[cfg(target_os = "macos")]
        {
            return write_to_command("pbcopy", &[], text);
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            return write_to_command("wl-copy", &[], text)
                || write_to_command("xsel", &["--clipboard", "--input"], text)
                || spawn_clipboard_command("xclip", &["-selection", "clipboard"], text);
        }

        #[allow(unreachable_code)]
        false
    }

    pub fn read_from_clipboard() -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            return read_from_command(
                "powershell",
                &["-NoProfile", "-STA", "-Command", "Get-Clipboard -Raw"],
            )
            .or_else(|| {
                read_from_command(
                    "powershell.exe",
                    &["-NoProfile", "-STA", "-Command", "Get-Clipboard -Raw"],
                )
            });
        }

        #[cfg(target_os = "macos")]
        {
            return read_from_command("pbpaste", &[]);
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            return read_from_command("wl-paste", &["--no-newline"])
                .or_else(|| read_from_command("xclip", &["-selection", "clipboard", "-o"]))
                .or_else(|| read_from_command("xsel", &["--clipboard", "--output"]));
        }

        #[allow(unreachable_code)]
        None
    }
}

pub fn open_url(url: &str) -> bool {
    platform::open_url(url)
}

#[cfg(target_arch = "wasm32")]
pub fn open_and_copy(
    url: &str,
    success_message: &str,
    copy_failure_message: &str,
    copied_only_message: &str,
    total_failure_message: &str,
) {
    platform::open_and_copy(
        url,
        success_message,
        copy_failure_message,
        copied_only_message,
        total_failure_message,
    );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn copy_to_clipboard(text: &str) -> bool {
    platform::copy_to_clipboard(text)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_from_clipboard() -> Option<String> {
    platform::read_from_clipboard()
}

pub fn social_profile_url(selection_index: usize) -> Option<&'static str> {
    match selection_index {
        0 => Some("https://github.com/TheDeveloperMalek/"),
        1 => Some("https://www.linkedin.com/in/malekalset/"),
        _ => None,
    }
}

