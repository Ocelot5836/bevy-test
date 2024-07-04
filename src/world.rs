use bevy::math::IVec3;

pub struct RenderChunk {
    blocks: [i8; VoxelWorld::CHUNK_SIZE * VoxelWorld::CHUNK_SIZE * VoxelWorld::CHUNK_SIZE],
}

impl RenderChunk {
    fn create_solid(block: i8) -> Self {
        Self {
            blocks: [block; VoxelWorld::CHUNK_SIZE * VoxelWorld::CHUNK_SIZE * VoxelWorld::CHUNK_SIZE]
        }
    }

    fn get_block(&self, pos: IVec3) -> i8 {
        return self.blocks[(pos.x as usize & 15) + ((pos.y as usize & 15) + (pos.z as usize & 15) * VoxelWorld::CHUNK_SIZE) * VoxelWorld::CHUNK_SIZE];
    }

    fn set_block(&mut self, pos: IVec3, block: i8) {
        self.blocks[(pos.x as usize & 15) + ((pos.y as usize & 15) + (pos.z as usize & 15) * VoxelWorld::CHUNK_SIZE) * VoxelWorld::CHUNK_SIZE] = block;
    }
}

pub struct VoxelWorld {
    chunks: Vec<RenderChunk>,
    size: IVec3,
}

impl VoxelWorld {
    pub const CHUNK_SIZE: usize = 16;
    pub const AIR: i8 = 0;
    pub const STONE: i8 = 1;

    pub fn create(grid_size: i32) -> Self {
        let mut chunks: Vec<RenderChunk> = Vec::new();
        for _ in 0..grid_size * grid_size * grid_size {
            chunks.push(RenderChunk::create_solid(VoxelWorld::AIR));
        }
        Self {
            chunks,
            size: IVec3::splat(grid_size),
        }
    }

    pub fn get_chunk(&self, chunk_pos: IVec3) -> Option<&RenderChunk> {
        if chunk_pos.x < 0 || chunk_pos.x >= self.size.x || chunk_pos.y < 0 || chunk_pos.y >= self.size.y || chunk_pos.z < 0 || chunk_pos.z >= self.size.z {
            return None;
        }
        return Some(&self.chunks[(chunk_pos.x + (chunk_pos.y + chunk_pos.z * self.size.y) * self.size.x) as usize]);
    }

    pub fn set_block(&mut self, pos: IVec3, block: i8) {
        if pos.x < 0 || pos.x >= self.size.x * VoxelWorld::CHUNK_SIZE as i32 || pos.y < 0 || pos.y >= self.size.y * VoxelWorld::CHUNK_SIZE as i32 || pos.z < 0 || pos.z >= self.size.z * VoxelWorld::CHUNK_SIZE as i32 {
            return;
        }
        let chunk_pos = pos / VoxelWorld::CHUNK_SIZE as i32;
        return self.chunks[(chunk_pos.x + (chunk_pos.y + chunk_pos.z * self.size.y) * self.size.x) as usize].set_block(pos, block);
    }

    pub fn get_chunks(&self) -> ChunkIterator {
        ChunkIterator {
            pointer: 0,
            world: self,
        }
    }
}

impl BlockGetter for VoxelWorld {
    fn get_block(&self, pos: IVec3) -> i8 {
        if pos.x < 0 || pos.x >= self.size.x * VoxelWorld::CHUNK_SIZE as i32 || pos.y < 0 || pos.y >= self.size.y * VoxelWorld::CHUNK_SIZE as i32 || pos.z < 0 || pos.z >= self.size.z * VoxelWorld::CHUNK_SIZE as i32 {
            return VoxelWorld::AIR;
        }
        let chunk_pos = pos / VoxelWorld::CHUNK_SIZE as i32;
        return return self.get_chunk(chunk_pos).unwrap().get_block(pos);
    }
}

pub struct ChunkIterator<'a> {
    pointer: i32,
    world: &'a VoxelWorld,
}

impl<'a> Iterator for ChunkIterator<'a> {
    type Item = (IVec3, &'a RenderChunk);

    fn next(&mut self) -> Option<Self::Item> {
        let chunk_pos = IVec3::new(self.pointer % self.world.size.x, self.pointer / (self.world.size.x * self.world.size.y), self.pointer / self.world.size.x);
        return self.world.get_chunk(chunk_pos).and_then(|chunk| {
            Some((chunk_pos, chunk))
        });
    }
}

pub trait BlockGetter {
    fn get_block(&self, pos: IVec3) -> i8;

    fn should_render_block(&self, pos: IVec3) -> bool {
        return self.get_block(pos) == VoxelWorld::STONE;
    }

    fn should_render_face(&self, pos: IVec3, offset: IVec3) -> bool {
        if self.get_block(pos) == VoxelWorld::AIR {
            return false;
        }

        return self.get_block(pos + offset) == VoxelWorld::AIR;
    }
}