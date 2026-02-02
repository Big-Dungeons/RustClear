use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::entities::npc::InteractableNPC;
use crate::dungeon::room::room::Room;
use glam::ivec3;
use server::block::rotatable::Rotate;
use server::entity::entity_appearance::PlayerAppearance;
use server::World;

// Once done implementing all rooms,
// and like finalized how I want to do them, make this use enum_dispatch or something
pub trait RoomImplementation {
    fn discover(&mut self, room: &mut Room, world: &mut World<Dungeon>);
}

pub struct MobRoom;

impl RoomImplementation for MobRoom {
    fn discover(&mut self, room: &mut Room, world: &mut World<Dungeon>) {

    }
}

pub struct Puzzle;

impl RoomImplementation for Puzzle {
    fn discover(&mut self, room: &mut Room, world: &mut World<Dungeon>) {
        let mut position = room.get_world_block_position(ivec3(17, 69, 24)).as_dvec3();
        position.x += 0.5;
        position.z += 0.5;

        let yaw = 180.0.rotate(room.rotation);

        world.spawn_entity(
            position,
            yaw,
            0.0,
            PlayerAppearance::new(
                Default::default(),
                "ewogICJ0aW1lc3RhbXAiIDogMTYxODc4MTA4Mzk0NywKICAicHJvZmlsZUlkIiA6ICJhNzdkNmQ2YmFjOWE0NzY3YTFhNzU1NjYxOTllYmY5MiIsCiAgInByb2ZpbGVOYW1lIiA6ICIwOEJFRDUiLAogICJzaWduYXR1cmVSZXF1aXJlZCIgOiB0cnVlLAogICJ0ZXh0dXJlcyIgOiB7CiAgICAiU0tJTiIgOiB7CiAgICAgICJ1cmwiIDogImh0dHA6Ly90ZXh0dXJlcy5taW5lY3JhZnQubmV0L3RleHR1cmUvOWI1Njg5NWI5NjU5ODk2YWQ2NDdmNTg1OTkyMzhhZjUzMmQ0NmRiOWMxYjAzODliOGJiZWI3MDk5OWRhYjMzZCIsCiAgICAgICJtZXRhZGF0YSIgOiB7CiAgICAgICAgIm1vZGVsIiA6ICJzbGltIgogICAgICB9CiAgICB9CiAgfQp9",
                "aNIhT2Tj20v1lONBOK3fIwBqJwWnjErq20h663Gb+PVmR9Iweh1h2ZEJ2pwDDnM4Af1XFDA5hS1Z9yOc8EdVTKyyi1yj9EIvMwQz/Q4N2sBsjWGZtCe8/Zy+X82iv0APB4cumE2gkgDbPjxCFNbpVKmV3U1WzwY/GKOMHofhWS1ULedQ1TszuMmDuHPLEzWaXigZ+xt5zChXvE8QoLTfBvgb8wtqVpyxAKf/o8xQduKiNE7t+de1CwOhLqbVTGh7DU0vLC5stDuqN+nC9dS7c2CG0ori6gFoGMvP4oIss6zm1nb0laMrZidJTgmuXk2Pv4NGDBXdYcAzhfWcSWGsBVMWrJfccgFheG+YcGYaYj6V2nBp0YTqqhN4wDt3ltyTNEMOr/JKyBTLzq/F7IL6rrdyMw+MbAgCa1FhfXxtzdQE2KsL55pbr2DZ8J4DYf+/OC1pWCJ4vvA/A1qGHyi3Zwtj9lCl1Jq5Qm2P9BgWxpk0ikJefRPMg4qWOEcYnjqwXuEp+IgTJi1xr+j/+g28aS1TsF8ijaJjSbEN4urrf3RYL+PZBcggzX9VaPB0NPdioOXznIotY+S6ZW7FnSh6UnrGAKadQBVLey5zmVWMfXlBUq9JMh0csuNd4dDQCLNK8oGORhMgksOMHhVaBie4otUgJ7ThR/WPjOAKiG2TNU0=",
            ),
            InteractableNPC {
                default_yaw: yaw,
                default_pitch: 0.0,
                interact_callback: |player| {
                },
            }
        );

        let mut position = room.get_world_block_position(ivec3(15, 69, 25)).as_dvec3();
        position.x += 0.5;
        position.z += 0.5;

        world.spawn_entity(
            position,
            yaw,
            0.0,
            PlayerAppearance::new(
                Default::default(),
                "ewogICJ0aW1lc3RhbXAiIDogMTYxODc4MTA4Mzk0NywKICAicHJvZmlsZUlkIiA6ICJhNzdkNmQ2YmFjOWE0NzY3YTFhNzU1NjYxOTllYmY5MiIsCiAgInByb2ZpbGVOYW1lIiA6ICIwOEJFRDUiLAogICJzaWduYXR1cmVSZXF1aXJlZCIgOiB0cnVlLAogICJ0ZXh0dXJlcyIgOiB7CiAgICAiU0tJTiIgOiB7CiAgICAgICJ1cmwiIDogImh0dHA6Ly90ZXh0dXJlcy5taW5lY3JhZnQubmV0L3RleHR1cmUvOWI1Njg5NWI5NjU5ODk2YWQ2NDdmNTg1OTkyMzhhZjUzMmQ0NmRiOWMxYjAzODliOGJiZWI3MDk5OWRhYjMzZCIsCiAgICAgICJtZXRhZGF0YSIgOiB7CiAgICAgICAgIm1vZGVsIiA6ICJzbGltIgogICAgICB9CiAgICB9CiAgfQp9",
                "aNIhT2Tj20v1lONBOK3fIwBqJwWnjErq20h663Gb+PVmR9Iweh1h2ZEJ2pwDDnM4Af1XFDA5hS1Z9yOc8EdVTKyyi1yj9EIvMwQz/Q4N2sBsjWGZtCe8/Zy+X82iv0APB4cumE2gkgDbPjxCFNbpVKmV3U1WzwY/GKOMHofhWS1ULedQ1TszuMmDuHPLEzWaXigZ+xt5zChXvE8QoLTfBvgb8wtqVpyxAKf/o8xQduKiNE7t+de1CwOhLqbVTGh7DU0vLC5stDuqN+nC9dS7c2CG0ori6gFoGMvP4oIss6zm1nb0laMrZidJTgmuXk2Pv4NGDBXdYcAzhfWcSWGsBVMWrJfccgFheG+YcGYaYj6V2nBp0YTqqhN4wDt3ltyTNEMOr/JKyBTLzq/F7IL6rrdyMw+MbAgCa1FhfXxtzdQE2KsL55pbr2DZ8J4DYf+/OC1pWCJ4vvA/A1qGHyi3Zwtj9lCl1Jq5Qm2P9BgWxpk0ikJefRPMg4qWOEcYnjqwXuEp+IgTJi1xr+j/+g28aS1TsF8ijaJjSbEN4urrf3RYL+PZBcggzX9VaPB0NPdioOXznIotY+S6ZW7FnSh6UnrGAKadQBVLey5zmVWMfXlBUq9JMh0csuNd4dDQCLNK8oGORhMgksOMHhVaBie4otUgJ7ThR/WPjOAKiG2TNU0=",
            ),
            InteractableNPC {
                default_yaw: yaw,
                default_pitch: 0.0,
                interact_callback: |player| {
                },
            }
        );

        let mut position = room.get_world_block_position(ivec3(13, 69, 24)).as_dvec3();
        position.x += 0.5;
        position.z += 0.5;

        world.spawn_entity(
            position,
            yaw,
            0.0,
            PlayerAppearance::new(
                Default::default(),
                "ewogICJ0aW1lc3RhbXAiIDogMTYxODc4MTA4Mzk0NywKICAicHJvZmlsZUlkIiA6ICJhNzdkNmQ2YmFjOWE0NzY3YTFhNzU1NjYxOTllYmY5MiIsCiAgInByb2ZpbGVOYW1lIiA6ICIwOEJFRDUiLAogICJzaWduYXR1cmVSZXF1aXJlZCIgOiB0cnVlLAogICJ0ZXh0dXJlcyIgOiB7CiAgICAiU0tJTiIgOiB7CiAgICAgICJ1cmwiIDogImh0dHA6Ly90ZXh0dXJlcy5taW5lY3JhZnQubmV0L3RleHR1cmUvOWI1Njg5NWI5NjU5ODk2YWQ2NDdmNTg1OTkyMzhhZjUzMmQ0NmRiOWMxYjAzODliOGJiZWI3MDk5OWRhYjMzZCIsCiAgICAgICJtZXRhZGF0YSIgOiB7CiAgICAgICAgIm1vZGVsIiA6ICJzbGltIgogICAgICB9CiAgICB9CiAgfQp9",
                "aNIhT2Tj20v1lONBOK3fIwBqJwWnjErq20h663Gb+PVmR9Iweh1h2ZEJ2pwDDnM4Af1XFDA5hS1Z9yOc8EdVTKyyi1yj9EIvMwQz/Q4N2sBsjWGZtCe8/Zy+X82iv0APB4cumE2gkgDbPjxCFNbpVKmV3U1WzwY/GKOMHofhWS1ULedQ1TszuMmDuHPLEzWaXigZ+xt5zChXvE8QoLTfBvgb8wtqVpyxAKf/o8xQduKiNE7t+de1CwOhLqbVTGh7DU0vLC5stDuqN+nC9dS7c2CG0ori6gFoGMvP4oIss6zm1nb0laMrZidJTgmuXk2Pv4NGDBXdYcAzhfWcSWGsBVMWrJfccgFheG+YcGYaYj6V2nBp0YTqqhN4wDt3ltyTNEMOr/JKyBTLzq/F7IL6rrdyMw+MbAgCa1FhfXxtzdQE2KsL55pbr2DZ8J4DYf+/OC1pWCJ4vvA/A1qGHyi3Zwtj9lCl1Jq5Qm2P9BgWxpk0ikJefRPMg4qWOEcYnjqwXuEp+IgTJi1xr+j/+g28aS1TsF8ijaJjSbEN4urrf3RYL+PZBcggzX9VaPB0NPdioOXznIotY+S6ZW7FnSh6UnrGAKadQBVLey5zmVWMfXlBUq9JMh0csuNd4dDQCLNK8oGORhMgksOMHhVaBie4otUgJ7ThR/WPjOAKiG2TNU0=",
            ),
            InteractableNPC {
                default_yaw: yaw,
                default_pitch: 0.0,
                interact_callback: |player| {
                },
            }
        );
    }
}