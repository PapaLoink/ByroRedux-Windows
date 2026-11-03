//! Prepare authored blend states from loaded content, before visibility filtering.

use byroredux_core::ecs::{Material, ParticleEmitter, World};
use byroredux_renderer::vulkan::context::is_refractive_material;

use crate::components::AlphaBlend;

pub(crate) fn loaded_blend_pipeline_keys(world: &World) -> Vec<(u8, u8, bool, bool)> {
    let mut keys = Vec::new();
    if let Some(blends) = world.query::<AlphaBlend>() {
        let materials = world.query::<Material>();
        for (entity, blend) in blends.iter() {
            let material = materials.as_ref().and_then(|q| q.get(entity));
            let kind = material.map_or(0, |m| m.material_kind);
            let refraction = material
                .and_then(|m| m.shader_type_fields.as_deref())
                .and_then(|f| f.multi_layer_refraction_scale)
                .unwrap_or(0.0);
            keys.push((
                blend.src_blend,
                blend.dst_blend,
                material.is_some_and(|m| m.wireframe),
                is_refractive_material(kind, refraction),
            ));
        }
    }
    if let Some(emitters) = world.query::<ParticleEmitter>() {
        for (_, emitter) in emitters.iter() {
            // Billboards always use filled effect geometry, including while
            // dormant. Do not wait for a live particle to reveal this state.
            keys.push((emitter.src_blend, emitter.dst_blend, false, false));
        }
    }
    keys.sort_unstable();
    keys.dedup();
    keys
}

pub(crate) fn prepare_loaded_blend_pipelines(
    world: &World,
    renderer: &mut byroredux_renderer::VulkanContext,
) -> anyhow::Result<bool> {
    let keys = loaded_blend_pipeline_keys(world);
    renderer.request_blend_preparation(&keys)
}

impl crate::App {
    /// Yield while compiling; winit continues dispatching native events.
    pub(crate) fn poll_blend_preparation(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        discover: bool,
    ) -> bool {
        let ctx = self.renderer.as_mut().unwrap();
        let was_pending = ctx.blend_preparation_pending();
        if !discover && !was_pending {
            return false;
        }
        let ready = if discover {
            prepare_loaded_blend_pipelines(&self.world, ctx)
        } else {
            ctx.request_blend_preparation(&[])
        };
        match ready {
            Ok(false) => {
                self.shader_restore_capture |= self.world
                    .resource::<crate::components::InputState>().mouse_captured;
                self.release_world_input_for_ui();
                self.last_frame = std::time::Instant::now();
                if let Some(win) = self.window.as_ref() {
                    win.set_title("ByroRedux | Preparing shaders...");
                }
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    std::time::Instant::now() + std::time::Duration::from_millis(16),
                ));
                true
            }
            Ok(true) => {
                if discover && !self.initial_blend_preparation_complete {
                    self.initial_blend_preparation_complete = true;
                    log::info!("Engine ready — entering game loop");
                }
                if was_pending {
                    self.last_frame = std::time::Instant::now();
                    log::info!("Loaded-scene blend preparation complete — resuming game loop");
                    if let Some(win) = self.window.as_ref() {
                        win.set_title("ByroRedux");
                        let size = win.inner_size();
                        let ctx = self.renderer.as_mut().unwrap();
                        let extent = ctx.frame_extents.output;
                        if size.width > 0
                            && size.height > 0
                            && [extent.width, extent.height] != [size.width, size.height]
                        {
                            if let Err(error) = ctx.recreate_swapchain([size.width, size.height]) {
                                log::error!("Deferred resize failed: {error:#}");
                                event_loop.exit();
                                return true;
                            }
                        }
                    }
                    if let Some(active) = self
                        .world
                        .try_resource::<byroredux_core::ecs::ActiveCamera>()
                    {
                        if let Some(win) = self.window.as_ref() {
                            let size = win.inner_size();
                            if size.width > 0 && size.height > 0 {
                                if let Some(mut cameras) =
                                    self.world.query_mut::<byroredux_core::ecs::Camera>()
                                {
                                    if let Some(camera) = cameras.get_mut(active.0) {
                                        camera.aspect = size.width as f32 / size.height as f32;
                                    }
                                }
                            }
                        }
                    }
                    if std::mem::take(&mut self.shader_restore_capture)
                        && !self.loading_screen.active()
                        && self.window.as_ref().is_some_and(|win| win.has_focus())
                    {
                        self.capture_world_input();
                    }
                    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
                }
                false
            }
            Err(error) => {
                log::error!("Loaded-scene blend preparation failed: {error:#}");
                event_loop.exit();
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preparation_rescans_new_cells_and_material_edits() {
        let mut world = World::new();
        assert!(loaded_blend_pipeline_keys(&world).is_empty());
        let entity = world.spawn();
        world.insert(
            entity,
            AlphaBlend {
                src_blend: 6,
                dst_blend: 7,
            },
        );
        assert_eq!(
            loaded_blend_pipeline_keys(&world),
            vec![(6, 7, false, false)]
        );
        world.insert(
            entity,
            Material {
                material_kind: byroredux_renderer::MATERIAL_KIND_GLASS,
                wireframe: true,
                ..Material::default()
            },
        );
        assert_eq!(loaded_blend_pipeline_keys(&world), vec![(6, 7, true, true)]);
        let streamed = world.spawn();
        world.insert(
            streamed,
            AlphaBlend {
                src_blend: 6,
                dst_blend: 0,
            },
        );
        assert_eq!(
            loaded_blend_pipeline_keys(&world),
            vec![(6, 0, false, false), (6, 7, true, true)]
        );
    }

    #[test]
    fn preparation_includes_hidden_meshes_and_deduplicates_authored_states() {
        let mut world = World::new();
        for _ in 0..2 {
            let entity = world.spawn();
            world.insert(
                entity,
                AlphaBlend {
                    src_blend: 6,
                    dst_blend: 7,
                },
            );
        }
        let glass = world.spawn();
        world.insert(
            glass,
            AlphaBlend {
                src_blend: 6,
                dst_blend: 7,
            },
        );
        world.insert(
            glass,
            Material {
                material_kind: byroredux_renderer::MATERIAL_KIND_GLASS,
                wireframe: true,
                ..Material::default()
            },
        );
        // No camera, transform or visibility is needed to discover either key.
        assert_eq!(
            loaded_blend_pipeline_keys(&world),
            vec![(6, 7, false, false), (6, 7, true, true)]
        );
    }

    #[test]
    fn preparation_includes_dormant_emitters_before_their_first_particle() {
        let mut world = World::new();
        let entity = world.spawn();
        let mut emitter = ParticleEmitter::torch_flame();
        emitter.src_blend = 6;
        emitter.dst_blend = 0;
        assert!(emitter.particles.is_empty());
        world.insert(entity, emitter);
        assert_eq!(
            loaded_blend_pipeline_keys(&world),
            vec![(6, 0, false, false)]
        );
    }
}
