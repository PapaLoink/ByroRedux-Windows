//! Single cancellable non-spatial soundtrack for original game movies.
use super::*;

impl AudioWorld {
    pub fn play_presentation(&mut self, sound: Sound) -> Result<(), String> {
        self.stop_presentation();
        let manager = self.manager.as_mut().ok_or("audio device unavailable")?;
        self.presentation = Some(manager.play(sound).map_err(|e| e.to_string())?);
        Ok(())
    }

    pub fn presentation_position(&self) -> Option<f64> {
        self.presentation.as_ref().filter(|handle| handle.state() != PlaybackState::Stopped).map(|handle| handle.position())
    }

    pub fn stop_presentation(&mut self) {
        if let Some(mut handle) = self.presentation.take() {
            handle.stop(Tween { duration: Duration::ZERO, ..Tween::default() });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presentation_can_be_stopped_without_an_audio_device() {
        let mut audio = AudioWorld::headless();
        audio.stop_presentation();
        assert_eq!(audio.presentation_position(), None);
    }
}
