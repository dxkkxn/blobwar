//! rust alpha - beta implementation for the blobwar game.
#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod board;
pub mod configuration;
pub(crate) mod positions;
pub(crate) mod shmem;
pub mod strategy;

#[cfg(test)]
mod tests {
    #[test]
    fn test_addition() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_subtraction() {
        assert_eq!(5 - 2, 3);
    }
}
