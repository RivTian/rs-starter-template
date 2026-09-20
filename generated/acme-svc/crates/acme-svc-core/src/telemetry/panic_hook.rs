use std::panic::PanicHookInfo;

/// Routes panics through `tracing` (with a backtrace) so they land in the same place as every
/// other log line. Falls back to the default hook while no subscriber is installed yet.
pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if !tracing::event_enabled!(tracing::Level::ERROR) {
            default_hook(info);
            return;
        }
        let location = info.location().map_or_else(
            || "unknown".to_owned(),
            |l| format!("{}:{}:{}", l.file(), l.line(), l.column()),
        );
        let thread = std::thread::current();
        let backtrace = std::backtrace::Backtrace::force_capture();
        tracing::error!(
            target: "panic",
            payload = payload_text(info),
            %location,
            thread = thread.name().unwrap_or("<unnamed>"),
            %backtrace,
            "panic"
        );
    }));
}

fn payload_text<'a>(info: &'a PanicHookInfo<'_>) -> &'a str {
    let payload = info.payload();
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("<non-string panic payload>")
}
