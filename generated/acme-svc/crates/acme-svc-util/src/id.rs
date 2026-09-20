//! Identifier strategy, in one place, so it can change in one place.

use uuid::Uuid;

/// A time-ordered UUID (version 7): sortable by creation time and safe as a primary key.
#[must_use]
pub fn new_v7() -> Uuid {
    Uuid::now_v7()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_v7() {
        let a = new_v7();
        let b = new_v7();
        assert_ne!(a, b);
        assert_eq!(a.get_version_num(), 7);
    }
}
