use crate::{
    OffsetInfo,
    PROVIDER_INSTANCE,
};

#[macro_export]
macro_rules! runtime_offset {
    ($default_value:expr, $module:expr, $class_name:expr, $class_member:expr) => {{
        static RESOLVED_OFFSET: ::std::sync::OnceLock<u64> = ::std::sync::OnceLock::new();

        *RESOLVED_OFFSET.get_or_init(|| {
            $crate::resolve_offset(&$crate::OffsetInfo {
                default_value: $default_value,
                module: $module,
                class_name: $class_name,
                member: $class_member,
            })
        })
    }};
}

pub fn resolve_offset(offset: &OffsetInfo) -> u64 {
    log::trace!(
        "Resolving offset {}::{}.{}",
        offset.module,
        offset.class_name,
        offset.member
    );
    let instance = PROVIDER_INSTANCE.read().unwrap();
    let Some(instance) = instance.as_ref() else {
        panic!("no schema provider set");
    };

    let Some(value) = instance.resolve_offset(offset) else {
        /*
         * The field is not present in the current game schema (e.g. renamed or removed by an update).
         * Fall back to the offset the bindings were generated with instead of aborting the process.
         */
        log::warn!(
            "Could not resolve offset for {}::{}.{}. Falling back to the generated offset 0x{:X} which might be outdated.",
            offset.module,
            offset.class_name,
            offset.member,
            offset.default_value
        );
        return offset.default_value;
    };

    log::trace!(" -> 0x{:X}", value);
    value
}
