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

    const BIT_WIDTH: usize = 16;
    pub fn is_air(mut block: u64) -> bool {
        let mut answer = true;
        for _ in 0..Self::BIT_WIDTH {
            if block & 1 == 1 {
                answer = false;
                break;
            }
            block >>= 1;
        }
        answer
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
