#![allow(dead_code)]

pub const HOTBAR_SIZE: usize = 10;
pub struct Inventory {
    hotbar: [Option<Item>; HOTBAR_SIZE],
    backpack: [Option<Item>; 32],
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            hotbar: [Option::None; HOTBAR_SIZE],
            backpack: [Option::None; 32],
        }
    }

    pub fn pick_item(&mut self, item: Item) -> bool {
        for cell in &mut self.hotbar {
            if cell.is_none() {
                *cell = Some(item);
                return true;
            }
        }

        for cell in &mut self.backpack {
            if cell.is_none() {
                *cell = Some(item);
                return true;
            }
        }
        false
    }

    pub fn get_item_from_hotbar(&mut self, position: usize) -> Option<Item> {
        assert!(position < self.hotbar.len());
        match &mut self.hotbar[position] {
            Some(item) => {
                // match &mut item.count {
                //     Count::Infinite => (),
                //     Count::Finite(remains) => {
                //         *remains -= 1;
                //         if *remains == 0 {
                //             self.hotbar[position] = None;
                //         }
                //     }
                // }
                Some(*item)
            }
            None => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Item {
    id: u64,
    type_: ItemType,
    count: Count,
}

impl Item {
    pub fn from_block(block: u64, total: Count) -> Self {
        Item {
            id: block,
            type_: ItemType::BLOCK,
            count: total,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Clone, Copy)]
pub enum ItemType {
    BLOCK,
    DECORATION,
}

#[derive(Clone, Copy)]
pub enum Count {
    Infinite,
    Finite(usize),
}
