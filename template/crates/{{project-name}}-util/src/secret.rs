//! A wrapper that keeps credentials out of logs, `Debug` output and rendered configuration.

use core::fmt;

use serde::{Deserialize, Serialize, Serializer};

/// A value that must never be printed. Access it deliberately with [`Secret::expose`].
///
/// * `Debug` / `Display` print `***`.
/// * `Serialize` writes `***` — good for `--print-config`, never for round-tripping.
/// * `Deserialize` is transparent, so config files and env vars hold the plain value.
#[derive(Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(transparent)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// Wraps a value.
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Returns the protected value. Make the call site obvious.
    #[must_use]
    pub const fn expose(&self) -> &T {
        &self.0
    }

    /// Unwraps the protected value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

impl<T> Serialize for Secret<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("***")
    }
}

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_leaks_through_debug_display_or_serialize() {
        let s = Secret::new(String::from("hunter2"));
        assert_eq!(format!("{s:?}"), "Secret(***)");
        assert_eq!(s.to_string(), "***");
        assert_eq!(serde_json::to_string(&s).unwrap_or_default(), "\"***\"");
        assert_eq!(s.expose(), "hunter2");
    }

    #[test]
    fn deserializes_transparently() {
        let s: Secret<String> = serde_json::from_str("\"plain\"").unwrap_or_default();
        assert_eq!(s.into_inner(), "plain");
    }
}
