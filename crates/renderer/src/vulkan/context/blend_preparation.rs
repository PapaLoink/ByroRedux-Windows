//! Off-thread blend compilation. The host yields instead of drawing while pending.
use super::{helpers, pipeline, VulkanContext};
use anyhow::{Context, Result};
use ash::vk;
use std::thread::JoinHandle;

type Key = (u8, u8, bool, bool);

// Own every worker-created handle, including partial successes on error/panic.
// VulkanContext joins before destroying the shared layout, render pass or device.
pub(super) struct CompiledBlends {
    device: ash::Device,
    cache: vk::PipelineCache,
    pipelines: Vec<(Key, vk::Pipeline)>,
}

impl Drop for CompiledBlends {
    fn drop(&mut self) {
        // SAFETY: these pipelines have never been submitted. The context keeps
        // the device alive until the worker and this result have been drained.
        unsafe {
            for (_, pipe) in self.pipelines.drain(..) {
                self.device.destroy_pipeline(pipe, None);
            }
            self.device.destroy_pipeline_cache(self.cache, None);
        }
    }
}

pub(super) type BlendPreparation = JoinHandle<Result<CompiledBlends>>;

fn missing_keys(
    keys: &[Key],
    wireframe_supported: bool,
    mut cached: impl FnMut(&Key) -> bool,
) -> Vec<Key> {
    let mut missing: Vec<_> = keys
        .iter()
        .map(|&(s, d, w, g)| (s, d, w && wireframe_supported, g))
        .filter(|key| !cached(key))
        .collect();
    missing.sort_unstable();
    missing.dedup();
    missing
}

// Opt-in, bounded engineering probe: exercise native event handling while the
// worker is outstanding, even when the driver cache makes compilation instant.
fn diagnostic_hold_duration(value: Option<&str>) -> std::time::Duration {
    std::time::Duration::from_millis(value.and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0).min(120_000))
}

fn preparation_running<T>(job: &Option<JoinHandle<T>>) -> bool {
    job.as_ref().is_some_and(|job| !job.is_finished())
}

impl VulkanContext {
    /// Final guard for generated draws and material edits after scene discovery.
    pub(super) fn prepare_draw_blend_states(
        &mut self,
        draws: &[super::DrawCommand],
    ) -> Result<bool> {
        let keys: Vec<_> = draws
            .iter()
            .filter(|draw| draw.alpha_blend)
            .map(|draw| {
                (
                    draw.src_blend,
                    draw.dst_blend,
                    draw.wireframe,
                    super::frame_params::is_refractive_glass(draw),
                )
            })
            .collect();
        self.request_blend_preparation(&keys)
    }

    pub fn blend_preparation_pending(&self) -> bool {
        self.blend_preparation.is_some()
    }

    /// Poll completion and enqueue missing authored states without blocking.
    /// Returns false until all requested variants can be used for drawing.
    /// The host must yield simulation/draw/resize while false and poll again.
    pub fn request_blend_preparation(&mut self, keys: &[Key]) -> Result<bool> {
        if preparation_running(&self.blend_preparation) {
            return Ok(false);
        }
        self.finish_blend_preparation()?;
        let missing = missing_keys(
            keys,
            self.device_caps.fill_mode_non_solid_supported,
            |key| self.blend_pipeline_cache.contains_key(key),
        );
        if missing.is_empty() {
            return Ok(true);
        }
        // A private cache avoids concurrent access to the main driver's cache.
        // SAFETY: main cache is live and accessed only on this thread.
        let seed = unsafe { self.device.get_pipeline_cache_data(self.pipeline_cache) }
            .context("read blend preparation cache")?;
        let cache = unsafe {
            self.device.create_pipeline_cache(
                &vk::PipelineCacheCreateInfo::default().initial_data(&seed),
                None,
            )
        }
        .context("create blend preparation cache")?;
        let mut result = CompiledBlends {
            device: self.device.clone(),
            cache,
            pipelines: Vec::new(),
        };
        let render_pass = self.swapchain.render_pass;
        let pipeline_layout = self.pipeline_layout;
        let extent = self.frame_extents.render;
        self.blend_preparation = Some(
            std::thread::Builder::new()
                .name("blend-pipeline-preparation".into())
                .spawn(move || {
                    let hold = diagnostic_hold_duration(
                        std::env::var("BYROREDUX_DIAGNOSTIC_BLEND_HOLD_MS").ok().as_deref());
                    if !hold.is_zero() {
                        log::warn!("DIAGNOSTIC ONLY: holding blend preparation worker for {} ms", hold.as_millis());
                        std::thread::sleep(hold);
                        log::warn!("DIAGNOSTIC ONLY: blend preparation worker hold ended");
                    }
                    for key @ (src, dst, wireframe, preserve_opaque_gbuffer) in missing {
                        log::warn!("Blend pipeline compile BEGIN {key:?} (worker)");
                        let started = std::time::Instant::now();
                        let pipe = pipeline::create_blend_pipeline(
                            pipeline::BlendPipelineCtx {
                                device: &result.device,
                                render_pass,
                                extent,
                                pipeline_cache: result.cache,
                                pipeline_layout,
                            },
                            src,
                            dst,
                            wireframe,
                            preserve_opaque_gbuffer,
                        )?;
                        result.pipelines.push((key, pipe));
                        helpers::save_pipeline_cache_if_grown(&result.device, result.cache);
                        log::info!(
                            "Blend pipeline compile END {key:?} (worker): {:.3} ms",
                            started.elapsed().as_secs_f64() * 1000.0
                        );
                    }
                    Ok(result)
                })
                .context("spawn blend preparation worker")?,
        );
        Ok(false)
    }

    // Only teardown / non-event-loop callers may wait here. Window resize is
    // deferred by the host; this safety barrier also protects other API users.
    pub(super) fn finish_blend_preparation(&mut self) -> Result<()> {
        let Some(job) = self.blend_preparation.take() else {
            return Ok(());
        };
        let mut compiled = job
            .join()
            .map_err(|_| anyhow::anyhow!("blend preparation worker panicked"))??;
        // SAFETY: join ended all worker cache access; both caches are live.
        unsafe {
            self.device
                .merge_pipeline_caches(self.pipeline_cache, &[compiled.cache])
        }
        .context("merge prepared blend cache")?;
        for (key, pipe) in compiled.pipelines.drain(..) {
            self.blend_pipeline_cache.insert(key, pipe);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_hold_is_opt_in_and_bounded() {
        assert!(diagnostic_hold_duration(None).is_zero());
        assert!(diagnostic_hold_duration(Some("invalid")).is_zero());
        assert_eq!(diagnostic_hold_duration(Some("30000")).as_millis(), 30_000);
        assert_eq!(diagnostic_hold_duration(Some("999999999")).as_millis(), 120_000);
    }

    #[test]
    fn preparation_gates_acquire_and_joins_before_gpu_teardown() {
        let draw = crate::source_scan::production_text(include_str!("draw.rs"));
        assert!(draw.find("self.prepare_draw_blend_states(draw_commands)").unwrap()
            < draw.find("self.sync_and_acquire_frame(&mut t)").unwrap());
        let teardown = crate::source_scan::production_text(include_str!("teardown.rs"));
        let drop_body = &teardown[teardown.find("impl Drop for VulkanContext").unwrap()..];
        assert!(drop_body.find("self.finish_blend_preparation()").unwrap()
            < drop_body.find("self.device.device_wait_idle()").unwrap());
        let resize = crate::source_scan::production_text(include_str!("resize.rs"));
        let resize_body = &resize[resize.find("pub fn recreate_swapchain(").unwrap()..];
        assert!(resize_body.find("self.finish_blend_preparation()").unwrap()
            < resize_body.find("self.recreate_swapchain_core(").unwrap());
    }

    #[test]
    fn pending_preparation_yields_and_completion_preserves_errors() {
        let (release, wait) = std::sync::mpsc::channel();
        let job = Some(std::thread::spawn(move || {
            wait.recv().unwrap();
            Err::<(), _>("synthetic compile failure")
        }));
        // The worker cannot finish until this thread releases it. Polling
        // must return rather than joining and deadlocking the event loop.
        assert!(preparation_running(&job));
        release.send(()).unwrap();
        assert_eq!(
            job.unwrap().join().unwrap(),
            Err("synthetic compile failure")
        );
        assert!(!preparation_running::<()>(&None));
    }

    #[test]
    fn preparation_discovers_later_states_and_normalizes_before_deduplication() {
        let cached = [(6, 7, false, false)];
        assert!(missing_keys(&cached, false, |k| cached.contains(k)).is_empty());
        let later = [
            (6, 7, true, false),
            (6, 0, true, false),
            (6, 0, false, false),
            (6, 7, false, true),
        ];
        assert_eq!(
            missing_keys(&later, false, |k| cached.contains(k)),
            vec![(6, 0, false, false), (6, 7, false, true)]
        );
    }
}
