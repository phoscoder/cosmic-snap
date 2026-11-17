//! Desktop notifications over the freedesktop D-Bus interface, which COSMIC's
//! notification daemon implements. Failures to notify are ignored on purpose:
//! the app must never break because a notification could not be shown.

use notify_rust::{Notification, Timeout, Urgency};
use std::path::Path;

const APP_NAME: &str = "Cosmic Snap";
const FALLBACK_ICON: &str = "camera-photo";

/// Absolute path of the installed app icon, or a generic camera icon name when it is not installed.
/// A full path is used so the notification does not depend on icon theme lookup.
fn default_icon() -> String {
    let data_home = std::env::var("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share"));
    [
        data_home.join("icons/hicolor/scalable/apps/cosmic-snap.svg"),
        "/usr/local/share/icons/hicolor/scalable/apps/cosmic-snap.svg".into(),
        "/usr/share/icons/hicolor/scalable/apps/cosmic-snap.svg".into(),
    ]
    .into_iter()
    .find(|p| p.exists())
    .map(|p| p.to_string_lossy().into_owned())
    .unwrap_or_else(|| FALLBACK_ICON.to_string())
}

fn send(summary: &str, body: &str, icon: &str, urgency: Urgency, timeout_ms: u32) {
    let mut n = Notification::new();
    n.appname(APP_NAME)
        .summary(summary)
        .body(body)
        .icon(icon)
        .urgency(urgency)
        .timeout(Timeout::Milliseconds(timeout_ms));
    let _ = n.show();
}

pub(crate) fn info(summary: &str, body: &str) {
    send(summary, body, &default_icon(), Urgency::Normal, 4000);
}

/// A saved screenshot: the file itself is used as the notification icon so it shows as a thumbnail.
pub(crate) fn saved(path: &Path) {
    send("Screenshot saved", &path.display().to_string(), &path.to_string_lossy(), Urgency::Normal, 6000);
}

pub(crate) fn error(summary: &str, body: &str) {
    send(summary, body, "dialog-error", Urgency::Critical, 8000);
}
