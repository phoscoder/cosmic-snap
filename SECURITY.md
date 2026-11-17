# Security Policy

## Supported versions

cosmic-snap is pre-1.0. Only the latest commit on `main` receives security fixes.

## Reporting a vulnerability

Please **do not** open a public issue for security problems.

Report privately through GitHub: open the repository's **Security** tab and choose
**Report a vulnerability** (private vulnerability reporting). Include what you found, how to reproduce it,
and the impact you expect.

You can expect an acknowledgement within a few days. Fixes are published on `main`, and you will be
credited in the release notes if you wish.

## Scope and things worth knowing

cosmic-snap captures your screen, so a few behaviors matter for privacy:

- Captures are written to a temporary directory only long enough to load them, then deleted.
- Saved screenshots go to `~/Pictures`; copied images go to the Wayland clipboard through `wl-copy`.
- **Redact draws an opaque black box.** It is intended to hide content, but double-check exported images
  before sharing anything sensitive.
- The app runs external programs (`cosmic-screenshot`, `wl-copy`, `tesseract`, `fc-match`) found on your
  `PATH`. It does not make network connections.
