const BLOCKS: [Block; 2] = [
    Block {
        name: BlockType::AIR,
        texutre_offset: 0,
    },
    Block {
        name: BlockType::DIRT,
        texutre_offset: 0,
    },
];

#[derive(Clone, Copy)]
pub enum BlockType {
    AIR,
    DIRT,
}

pub struct Block {
    name: BlockType,
    texutre_offset: usize,
}

impl Block {
    pub fn name(&self) -> BlockType {
        self.name
    }

    pub fn zoffset_texure(&self) -> usize {
        self.texutre_offset
    }
}

impl Clone for Block {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            texutre_offset: self.texutre_offset,
        }
    }
}

impl From<usize> for Block {
    fn from(id: usize) -> Self {
        BLOCKS[id].clone()
    }
}
