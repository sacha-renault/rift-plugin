use parking_lot::Once;

pub static INIT_LOG: Once = Once::new();

fn setup_logging_inner() -> Result<(), Box<dyn std::error::Error>> {
    use simplelog::*;
    use std::fs::File;

    std::panic::set_hook(Box::new(|info| {
        let msg = format!("{}", info);
        std::fs::write(
            r"C:\Users\alexi\Documents\Projects\plugin_test\panic.log",
            &msg,
        )
        .ok();
    }));

    CombinedLogger::init(vec![
        // File logger
        WriteLogger::new(
            LevelFilter::Info,
            ConfigBuilder::new()
                .add_filter_ignore(format!("{}", "vizia_core"))
                .set_target_level(LevelFilter::Info) // Shows module path
                .build(),
            File::create(r"C:\Users\alexi\Documents\Projects\plugin_test\app.log")?,
        ),
    ])?;
    Ok(())
}

pub fn setup_logging_once() {
    INIT_LOG.call_once(|| {
        let _ = setup_logging_inner();
    });
}
