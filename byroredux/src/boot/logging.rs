//! Default diagnostics; explicit RUST_LOG module directives take precedence.
pub(super) fn builder() -> env_logger::Builder {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    // Symphonia 0.6.1 repairs MP3 count1 padding overruns by discarding the
    // excess spectral quad and continuing. Some original music produces this
    // info diagnostic many times per second. Keep warnings/errors visible;
    // RUST_LOG=symphonia_bundle_mp3::layer3::requantize=info restores details.
    builder.filter_module("symphonia_bundle_mp3::layer3::requantize", log::LevelFilter::Warn);
    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovered_mp3_diagnostics_are_quiet_but_warnings_and_engine_info_remain() {
        let logger = builder().build();
        let enabled = |target, level| logger.matches(
            &log::Record::builder().target(target).level(level).args(format_args!("diagnostic")).build(),
        );
        let mp3 = "symphonia_bundle_mp3::layer3::requantize";
        assert!(!enabled(mp3, log::Level::Info));
        assert!(enabled(mp3, log::Level::Warn));
        assert!(enabled(mp3, log::Level::Error));
        assert!(enabled("symphonia_bundle_mp3::decoder", log::Level::Warn));
        assert!(enabled("byroredux::front_end", log::Level::Info));
    }

    #[test]
    fn explicit_module_filter_restores_mp3_diagnostics() {
        let logger = builder().parse_filters("symphonia_bundle_mp3::layer3::requantize=info").build();
        assert!(logger.matches(&log::Record::builder()
            .target("symphonia_bundle_mp3::layer3::requantize")
            .level(log::Level::Info).args(format_args!("diagnostic")).build()));
    }
}
