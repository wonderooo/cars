#[allow(clippy::large_enum_variant)]
pub mod copart;

#[macro_export]
macro_rules! impl_display_and_debug {
    ($type:ty, $body:expr) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                $body(self, f)
            }
        }

        impl std::fmt::Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                $body(self, f)
            }
        }
    };
}
