use std::thread::sleep;
use std::time::Duration;

use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, block_on, Task};
use bevy::tasks::futures_lite::future;

use crate::voxel_renderer::VoxelMaterial;

pub struct VoxelPlugin;

#[derive(Component)]
struct VoxelMeshTask(Task<CommandQueue>);

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VoxelMaterial>::default()).add_systems(Update, handle_tasks);
    }
}

fn handle_tasks(mut commands: Commands, mut transform_tasks: Query<&mut VoxelMeshTask>) {
    for mut task in &mut transform_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            // append the returned command queue to have it execute later
            commands.append(&mut commands_queue);
        }
    }
}

pub fn build_mesh(mut commands: Commands, pos: Vec3) {
    let thread_pool = AsyncComputeTaskPool::get();
    let entity = commands.spawn_empty().id();
    let task = thread_pool.spawn(async move {
        let mesh = long_task();

        let mut command_queue = CommandQueue::default();

        // we use a raw command queue to pass a FnOne(&mut World) back to be
        // applied in a deferred manner.
        command_queue.push(move |world: &mut World| {
            let mesh = world.get_resource_mut::<Assets<Mesh>>().unwrap().add(mesh);
            let material = world.get_resource_mut::<Assets<VoxelMaterial>>().unwrap().add(VoxelMaterial::default());
            world
                .entity_mut(entity)
                // Add our new PbrBundle of components to our tagged entity
                .insert(MaterialMeshBundle {
                    mesh,
                    material,
                    transform: Transform::from_translation(pos),
                    ..default()
                })
                // Task is complete, so remove task component from entity
                .remove::<VoxelMeshTask>();
        });

        command_queue
    });

    commands.entity(entity).insert(VoxelMeshTask(task));
}

fn long_task() -> Mesh {
    sleep(Duration::from_millis(1000));
    Plane3d::default().into()
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