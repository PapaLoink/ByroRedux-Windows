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
) -> anyhow::Result<()> {
    let keys = loaded_blend_pipeline_keys(world);
    log::info!(
        "Preparing loaded-scene blend pipelines: {} distinct states",
        keys.len()
    );
    renderer.prepare_blend_pipelines(&keys)
}

#[cfg(test)]
mod tests {
    use super::*;

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
