use crate::powder::chunk::ParticleData;
use crate::powder::local::LocalChunks;
use crate::powder::ParticleDefinition;
use glam::IVec2;
use std::mem;

pub fn simulate<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(
    local: &mut LocalChunks<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>,
    definitions: &[ParticleDefinition],
    center_particle: &mut ParticleData,
) {
    let definition = &definitions[center_particle.r#type];

    let center_position = center_particle.position;
    let left_position = center_position + IVec2::new(-1, 0);
    let right_position = center_position + IVec2::new(1, 0);
    let top_position = center_position + IVec2::new(0, 1);

    let center_type = center_particle.r#type;
    let mut center_hpressure = center_particle.hpressure;

    let mut left_particle: Option<&mut ParticleData> = unsafe { mem::transmute(local.get_particle_mut(left_position)) };
    let mut left_type = left_particle.as_ref().map(|p| p.r#type).unwrap_or(usize::MAX);
    let mut left_hpressure = left_particle.as_ref().map(|p| p.hpressure).unwrap_or(0.0);

    let mut right_particle: Option<&mut ParticleData> = unsafe { mem::transmute(local.get_particle_mut(right_position)) };
    let mut right_type = right_particle.as_ref().map(|p| p.r#type).unwrap_or(usize::MAX);
    let mut right_hpressure = right_particle.as_ref().map(|p| p.hpressure).unwrap_or(0.0);

    let mut top_particle: Option<&mut ParticleData> = unsafe { mem::transmute(local.get_particle_mut(top_position)) };
    let mut top_type = top_particle.as_ref().map(|p| p.r#type).unwrap_or(usize::MAX);
    let mut top_hpressure = top_particle.as_ref().map(|p| p.hpressure).unwrap_or(0.0);

    // ----------------------------------------------------------------
    // Inflate particle right and left by averagin hydrostatic pressure
    // ----------------------------------------------------------------

    let mut average_hpressure = center_hpressure;
    let mut average_count = 1;

    if left_type == usize::MAX || left_type == center_type {
        if left_type == center_type {
            average_hpressure += left_hpressure;
        }
        average_count += 1;
    }
    if right_type == usize::MAX || right_type == center_type {
        if right_type == center_type {
            average_hpressure += right_hpressure;
        }
        average_count += 1;
    }
    average_hpressure /= average_count as f32;

    if average_hpressure >= definition.extensibility {
        let particle = ParticleData {
            r#type: center_type,
            state: definition.state,
            color: definition.color,
            hpressure: average_hpressure,
            ..Default::default()
        };

        if left_type == usize::MAX {
            local.add_particle(left_position, particle);
            left_type = center_type;
            left_hpressure = average_hpressure;
        } else if left_type == center_type {
            left_hpressure = average_hpressure;
        }
        if right_type == usize::MAX {
            local.add_particle(right_position, particle);
            right_type = center_type;
            right_hpressure = average_hpressure;
        } else if left_type == center_type {
            right_hpressure = average_hpressure;
        }

        center_hpressure = average_hpressure;
    }

    // -------------------------------------------------------------------
    // Inflate particle top if hydrostatic pressure is above certain level
    // -------------------------------------------------------------------

    if top_type == center_type {
        let average = (center_hpressure + top_hpressure) / 2.0;
        center_hpressure = average + definition.compressibility / 2.0;
        top_hpressure = average - definition.compressibility / 2.0;

        // Try to fill center particle so the hydrostatic pressure is at least 1.0 + compressibility
        if center_hpressure < 1.0 + definition.compressibility {
            let diff = 1.0 + definition.compressibility - center_hpressure;
            if top_hpressure > diff {
                center_hpressure += diff;
                top_hpressure -= diff;
            } else {
                center_hpressure += top_hpressure;

                top_hpressure = 0.0;
                top_type = usize::MAX;
                local.remove_particle(top_position);
            }
        }
    // Average hydrostatic pressure when there's a free space above
    } else if top_type == usize::MAX && center_hpressure > 1.0 + definition.compressibility {
        center_hpressure /= 2.0;
        top_hpressure = center_hpressure;

        local.add_particle(
            top_position,
            ParticleData { r#type: center_type, state: definition.state, color: definition.color, hpressure: top_hpressure, ..Default::default() },
        );
        top_type = center_type;
    }

    // -----------------------------
    // Update particles with results
    // -----------------------------

    let center_hpressure_ratio = f32::min(1.0, center_hpressure / definition.hpressure_gradient_length);
    let center_color = definition.color * (1.0 - center_hpressure_ratio) + definition.hpressure_gradient_end * center_hpressure_ratio;

    center_particle.hpressure = center_hpressure;
    local.set_particle_color(center_position, center_color);

    if left_type == center_type {
        if let Some(particle) = &mut left_particle {
            let left_hpressure_ratio = f32::min(1.0, left_hpressure / definition.hpressure_gradient_length);
            let left_color = definition.color * (1.0 - left_hpressure_ratio) + definition.hpressure_gradient_end * left_hpressure_ratio;

            particle.hpressure = left_hpressure;
            local.set_particle_color(left_position, left_color);
        }
    }

    if right_type == center_type {
        if let Some(particle) = &mut right_particle {
            let right_hpressure_ratio = f32::min(1.0, right_hpressure / definition.hpressure_gradient_length);
            let right_color = definition.color * (1.0 - right_hpressure_ratio) + definition.hpressure_gradient_end * right_hpressure_ratio;

            particle.hpressure = right_hpressure;
            local.set_particle_color(right_position, right_color);
        }
    }

    if top_type == center_type {
        if let Some(particle) = &mut top_particle {
            let top_hpressure_ratio = f32::min(1.0, top_hpressure / definition.hpressure_gradient_length);
            let top_color = definition.color * (1.0 - top_hpressure_ratio) + definition.hpressure_gradient_end * top_hpressure_ratio;

            particle.hpressure = top_hpressure;
            local.set_particle_color(top_position, top_color);
        }
    }
}
