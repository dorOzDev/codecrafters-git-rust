pub enum EventName {
    CtrlC,
}

impl EventName {
    fn as_str(&self) -> &str {
        match self {
            EventName::CtrlC => "ctrlc",
        }
    }
}

/// Registers a handler for a given signal event, warning if it may override an existing handler.
/// For now, only supports "ctrlc" (SIGINT) as event_name.
pub fn register_signal_handler<F>(event_name: EventName, handler: F)
where
    F: Fn() + Send + 'static,
{
    // Warn the developer that this may override an existing handler.
    eprintln!(
        "Warning: Registering a handler for '{}' may override any previously set handler for this event.",
        event_name.as_str()
    );

    match event_name {
        _ctrl_c => {
            ctrlc::set_handler(move || {
                handler();
            }).expect("Error setting Ctrl-C handler");
        }
    }
}

