#[macro_export]
macro_rules! tr {
    ($key:expr) => {
        crate::common::i18n::tr($key, &[])
    };
    ($key:expr, $($k:ident = $v:expr),+ $(,)?) => {
        crate::common::i18n::tr(
            $key,
            &[$((stringify!($k), $v)),+]
        )
    };
}