use std::ops::Deref;
use std::sync::{Arc, RwLock};

use bevy::ecs::system::CommandQueue;
use bevy::pbr::{ExtendedMaterial, OpaqueRendererMethod};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::tasks::{AsyncComputeTaskPool, block_on, Task};
use bevy::tasks::futures_lite::future;

use crate::voxel_renderer::VoxelMaterial;
use crate::world::{BlockGetter, VoxelWorld};

pub struct VoxelPlugin;

#[derive(Component)]
struct VoxelMeshTask(Task<CommandQueue>, IVec3);

#[derive(Component)]
struct VoxelMesh {
    chunk_pos: IVec3,
}

#[derive(Resource)]
pub struct ClientWorld(pub Arc<RwLock<VoxelWorld>>);

impl ClientWorld {
    fn create(world: VoxelWorld) -> Self {
        Self {
            0: Arc::new(RwLock::new(world))
        }
    }
}

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        let mut world = VoxelWorld::create(1);
        world.set_block(IVec3::new(0, 1, 0), VoxelWorld::STONE);
        world.set_block(IVec3::new(4, 0, 0), VoxelWorld::STONE);
        world.set_block(IVec3::new(3, 0, 0), VoxelWorld::STONE);
        world.set_block(IVec3::new(2, 0, 0), VoxelWorld::STONE);
        world.set_block(IVec3::new(1, 0, 0), VoxelWorld::STONE);

        app.add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, VoxelMaterial>>::default())
            .add_systems(Update, handle_tasks)
            .insert_resource(ClientWorld::create(world));
    }
}

fn handle_tasks(mut commands: Commands, mut transform_tasks: Query<&mut VoxelMeshTask>, chunks: Query<(Entity, &VoxelMesh)>) {
    for mut task in &mut transform_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            // append the returned command queue to have it execute later
            commands.append(&mut commands_queue);

            for (entity, mesh) in chunks.iter() {
                if mesh.chunk_pos == task.1 {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

pub fn schedule(mut commands: Commands, voxel_world: Arc<RwLock<dyn BlockGetter>>, chunk_pos: IVec3) {
    let thread_pool = AsyncComputeTaskPool::get();
    let entity = commands.spawn_empty().id();

    let task = thread_pool.spawn_local(async move {
        let mesh = build_mesh(voxel_world.read().unwrap().deref(), chunk_pos * VoxelWorld::CHUNK_SIZE as i32);
        // let mesh = {
        //     let positions = vec![
        //         Vec3::new(16.0, 0.0, 0.0),
        //         Vec3::new(0.0, 0.0, 0.0),
        //         Vec3::new(0.0, 0.0, 16.0),
        //         Vec3::new(16.0, 0.0, 16.0),
        //     ];
        //     let normals = vec![Vec3::new(0.0, 1.0, 0.0); 4];
        //     let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3]);
        //
        //     Mesh::new(
        //         PrimitiveTopology::TriangleList,
        //         RenderAssetUsages::default(),
        //     ).with_inserted_indices(indices)
        //         .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        //         .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        // };

        let mut command_queue = CommandQueue::default();

        // we use a raw command queue to pass a FnOne(&mut World) back to be
        // applied in a deferred manner.
        command_queue.push(move |world: &mut World| {
            let mesh = world.get_resource_mut::<Assets<Mesh>>().unwrap().add(mesh);
            let material = world.get_resource_mut::<Assets<ExtendedMaterial<StandardMaterial, VoxelMaterial>>>().unwrap().add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color: Color::RED,
                    perceptual_roughness: 0.8,
                    // can be used in forward or deferred mode.
                    opaque_render_method: OpaqueRendererMethod::Auto,
                    // in deferred mode, only the PbrInput can be modified (uvs, color and other material properties),
                    // in forward mode, the output can also be modified after lighting is applied.
                    // see the fragment shader `extended_material.wgsl` for more info.
                    // Note: to run in deferred mode, you must also add a `DeferredPrepass` component to the camera and either
                    // change the above to `OpaqueRendererMethod::Deferred` or add the `DefaultOpaqueRendererMethod` resource.
                    ..Default::default()
                },
                extension: VoxelMaterial { quantize_steps: 20 },
            });

            world
                .entity_mut(entity)
                // Add our new PbrBundle of components to our tagged entity
                .insert((MaterialMeshBundle {
                    mesh,
                    material,
                    ..default()
                }, VoxelMesh {
                    chunk_pos
                }))
                // Task is complete, so remove task component from entity
                .remove::<VoxelMeshTask>();
        });

        command_queue
    });

    commands.entity(entity).insert(VoxelMeshTask(task, chunk_pos));
}

// DOWN(0, 1, -1, "down", Direction.AxisDirection.NEGATIVE, Direction.Axis.Y, new Vec3i(0, -1, 0)),
// UP(1, 0, -1, "up", Direction.AxisDirection.POSITIVE, Direction.Axis.Y, new Vec3i(0, 1, 0)),
// NORTH(2, 3, 2, "north", Direction.AxisDirection.NEGATIVE, Direction.Axis.Z, new Vec3i(0, 0, -1)),
// SOUTH(3, 2, 0, "south", Direction.AxisDirection.POSITIVE, Direction.Axis.Z, new Vec3i(0, 0, 1)),
// WEST(4, 5, 1, "west", Direction.AxisDirection.NEGATIVE, Direction.Axis.X, new Vec3i(-1, 0, 0)),
// EAST(5, 4, 3, "east", Direction.AxisDirection.POSITIVE, Direction.Axis.X, new Vec3i(1, 0, 0));

fn build_mesh(world: &dyn BlockGetter, start_pos: IVec3) -> Mesh {
    let mut positions: Vec<Vec3> = Vec::new();
    let mut normals: Vec<Vec3> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut count = 0;
    for z in 0..VoxelWorld::CHUNK_SIZE as i32 {
        for y in 0..VoxelWorld::CHUNK_SIZE as i32 {
            for x in 0..VoxelWorld::CHUNK_SIZE as i32 {
                let pos = start_pos + IVec3::new(x, y, z);
                if !world.should_render_block(pos) {
                    continue;
                }

                let mut rendered_count = 0;

                // UP
                if world.should_render_face(pos, IVec3::new(0, 1, 0)) {
                    positions.push(Vec3::new((x + 1) as f32, (y + 1) as f32, z as f32));
                    positions.push(Vec3::new(x as f32, (y + 1) as f32, z as f32));
                    positions.push(Vec3::new(x as f32, (y + 1) as f32, (z + 1) as f32));
                    positions.push(Vec3::new((x + 1) as f32, (y + 1) as f32, (z + 1) as f32));

                    normals.push(Vec3::new(0.0, 1.0, 0.0));
                    normals.push(Vec3::new(0.0, 1.0, 0.0));
                    normals.push(Vec3::new(0.0, 1.0, 0.0));
                    normals.push(Vec3::new(0.0, 1.0, 0.0));
                    rendered_count += 1;
                }

                // DOWN
                if world.should_render_face(pos, IVec3::new(0, -1, 0)) {
                    positions.push(Vec3::new(x as f32, y as f32, z as f32));
                    positions.push(Vec3::new((x + 1) as f32, y as f32, z as f32));
                    positions.push(Vec3::new((x + 1) as f32, y as f32, (z + 1) as f32));
                    positions.push(Vec3::new(x as f32, y as f32, (z + 1) as f32));

                    normals.push(Vec3::new(0.0, -1.0, 0.0));
                    normals.push(Vec3::new(0.0, -1.0, 0.0));
                    normals.push(Vec3::new(0.0, -1.0, 0.0));
                    normals.push(Vec3::new(0.0, -1.0, 0.0));
                    rendered_count += 1;
                }

                // EAST
                if world.should_render_face(pos, IVec3::new(1, 0, 0)) {
                    positions.push(Vec3::new((x + 1) as f32, y as f32, (z + 1) as f32));
                    positions.push(Vec3::new((x + 1) as f32, y as f32, z as f32));
                    positions.push(Vec3::new((x + 1) as f32, (y + 1) as f32, z as f32));
                    positions.push(Vec3::new((x + 1) as f32, (y + 1) as f32, (z + 1) as f32));

                    normals.push(Vec3::new(1.0, 0.0, 0.0));
                    normals.push(Vec3::new(1.0, 0.0, 0.0));
                    normals.push(Vec3::new(1.0, 0.0, 0.0));
                    normals.push(Vec3::new(1.0, 0.0, 0.0));
                    rendered_count += 1;
                }

                // WEST
                if world.should_render_face(pos, IVec3::new(-1, 0, 0)) {
                    positions.push(Vec3::new(x as f32, y as f32, z as f32));
                    positions.push(Vec3::new(x as f32, y as f32, (z + 1) as f32));
                    positions.push(Vec3::new(x as f32, (y + 1) as f32, (z + 1) as f32));
                    positions.push(Vec3::new(x as f32, (y + 1) as f32, z as f32));

                    normals.push(Vec3::new(-1.0, 0.0, 0.0));
                    normals.push(Vec3::new(-1.0, 0.0, 0.0));
                    normals.push(Vec3::new(-1.0, 0.0, 0.0));
                    normals.push(Vec3::new(-1.0, 0.0, 0.0));
                    rendered_count += 1;
                }

                // NORTH
                if world.should_render_face(pos, IVec3::new(0, 0, -1)) {
                    positions.push(Vec3::new(x as f32, y as f32, z as f32));
                    positions.push(Vec3::new(x as f32, (y + 1) as f32, z as f32));
                    positions.push(Vec3::new((x + 1) as f32, (y + 1) as f32, z as f32));
                    positions.push(Vec3::new((x + 1) as f32, y as f32, z as f32));

                    normals.push(Vec3::new(0.0, 0.0, -1.0));
                    normals.push(Vec3::new(0.0, 0.0, -1.0));
                    normals.push(Vec3::new(0.0, 0.0, -1.0));
                    normals.push(Vec3::new(0.0, 0.0, -1.0));
                    rendered_count += 1;
                }

                // SOUTH
                if world.should_render_face(pos, IVec3::new(0, 0, 1)) {
                    positions.push(Vec3::new((x + 1) as f32, y as f32, (z + 1) as f32));
                    positions.push(Vec3::new((x + 1) as f32, (y + 1) as f32, (z + 1) as f32));
                    positions.push(Vec3::new(x as f32, (y + 1) as f32, (z + 1) as f32));
                    positions.push(Vec3::new(x as f32, y as f32, (z + 1) as f32));

                    normals.push(Vec3::new(0.0, 0.0, 1.0));
                    normals.push(Vec3::new(0.0, 0.0, 1.0));
                    normals.push(Vec3::new(0.0, 0.0, 1.0));
                    normals.push(Vec3::new(0.0, 0.0, 1.0));
                    rendered_count += 1;
                }

                if rendered_count > 0 {
                    for _ in 0..rendered_count {
                        indices.push(count);
                        indices.push(count + 1);
                        indices.push(count + 2);
                        indices.push(count);
                        indices.push(count + 2);
                        indices.push(count + 3);
                        count += 4;
                    }
                }
            }
        }
    }

    // let positions = vec![
    //     Vec3::new(16.0, 0.0, 0.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.0, 0.0, 16.0),
    //     Vec3::new(16.0, 0.0, 16.0),
    // ];
    // let normals = vec![Vec3::new(0.0, 1.0, 0.0); 4];
    // let indices = Indices::U32(vec![0, 1, 2, 0, 2, 3]);

    return Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    ).with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
}

// fn create_voxel_mesh(mut task_executor: AsyncTaskRunner<Mesh>) {
//     task_executor.start(long_task());
//     match task_executor.poll() {
//         AsyncTaskStatus::Idle => {
//             println!("Started new task!");
//         }
//         AsyncTaskStatus::Pending => {
//             // <Insert loading screen>
//         }
//         AsyncTaskStatus::Finished(v) => {
//             println!("Received {v}");
//         }
//     }
// }