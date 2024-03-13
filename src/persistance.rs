use std::{borrow::BorrowMut, path::Path, sync::Arc};

use std::sync::{RwLock, RwLockReadGuard};



/*
persistence layer

A file contains 1 segment

A segment is a unit of a list of positions (256 bit numbers) and the corresponding offset id.

An offset id is unique to it's segment

offset_id is a 64bit number,

Each segment has a unique segment id. A segment id is unique to it's containing dataset.

Each dataset typically corresponds to a host or stand-alone disk


A unique position can be identified by the triplet (dataset_id, segment_id, offset_id)


Operations:

insert position

no need for delete or update really

fetch 256 bit position given id triplet

search for 256 bit position in database, return triplet id

segments are sorted before being stored. Segments can be merged or split.


 */

 // 256 bit
// pub enum PositionTrie
pub struct PositionTrieAddress {
    pub value: [u16; 16] // 16x 16bit matrix; each row = value of that PositionTrie's value
                        // The 16 rows form a path to a PositionTrieNode where is_terminal is True
                        // The combined bit values of the PositionTrie node values on the path up to this point
                        // is the position itself

}

pub struct PositionTrieConfig {
    children_size: usize,
}

#[derive(Default, Debug)]
pub struct PositionTrieNode<'a> {
    pub value: u16,
    pub children: Vec<RwLock<PositionTrieNode<'a>>>, // size up to 2^16
    pub are_children_dirty: bool,
    pub is_terminal: bool,
}

impl<'a> PositionTrieNode<'a> {
    pub fn new(value: u16, terminal: bool) -> Self {
        PositionTrieNode {
            value: value,
            children: Vec::<RwLock<PositionTrieNode>>::new(),
            are_children_dirty: false,
            is_terminal: terminal,
        }
    }

    pub fn add_child(&mut self, child: RwLock<PositionTrieNode<'a>>) -> &'a mut PositionTrieNode {
        self.children.push(child);
        self
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn get_node(&self, index: usize) -> Option<&'a RwLock<PositionTrieNode>> {
        self.children.get(index)
    }

    pub fn get_mut_node(&mut self, index: usize) -> Option<&'a mut RwLock<PositionTrieNode>> {
        self.children.get_mut(index)
    }

    // given a u16 value, find a PositionTrieNode with a matching value and return the node's index as OK(), or
    // return an ERR with the index that should have been there
    pub fn get_node_index_by_value(&self, value: u16) -> Result<usize, usize> {
        let mut index: usize = 0;
        for mutex in self.children.iter() {
            let node = mutex.read().unwrap();
            if node.value == value {
                return Ok(index);
            }
            index += 1;
        }
        Err(index)
    }
}

pub struct PositionTrie<'a> {
    root: RwLock<PositionTrieNode<'a>>,
}

impl<'a> PositionTrie<'_> {
    pub fn new() -> Self {
        PositionTrie {
            root: RwLock::new(PositionTrieNode::default()),
        }
    }
    /*
    pub fn new(value: u16, is_terminal: bool) -> Self {
        PositionTrieNode::default()
    }
    */
    /*
    pub fn level_stat(&self, level: usize) -> i32 {
        let mut node_count = 0;
        let mut cur_level = 0;
        let mut nodes_until_level = 0;
        let mut levels_count: [i32; 16] = [
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        let mut nodes = Vec::<RwLockReadGuard<PositionTrieNode>>::new();
        nodes.push(self.root.read().unwrap());

        while nodes.len() > 0 {
            let cur_node =  nodes.pop().unwrap().read().unwrap();
            for node in cur_node.children {
                nodes.push(node);
                nodes_until_level += 1;
            }
            node_count += 1;
        }

        node_count
    } */

    pub fn insert(&mut self, pos: &PositionTrieAddress) -> i32 {
        let mut current_node_lock = &self.root;

        let level_count = 16;
        let mut seen_levels = 0;

        for level in 0..(level_count - 1)  {
            //let mut maybe_node: Option<&mut PositionTrieNode> = None;
            let mut current_node: RwLockReadGuard<'_, PositionTrieNode<'_>> = current_node_lock.read().unwrap();
            current_node = match current_node.get_node_index_by_value(pos.value[level]) {
                Ok(idx) => {
                    seen_levels += 1;
                    //current_node.get_mut_node(idx).unwrap()
                    current_node.get_node(idx).unwrap().read().unwrap()
                }
                Err(idx2) => {
                    let newnode = PositionTrieNode::new(pos.value[level], level >= (level_count - 1));
                    let mut mut_cur_node = current_node_lock.write().unwrap();
                    mut_cur_node.add_child(RwLock::new(newnode));
                    current_node.get_node(idx2).unwrap().read().unwrap()
                }
            };
        }
        seen_levels
    }

    /*
    pub fn merge_children(&self, &newchildRen: Vec<PositionTrieNode>) {
        for child in self.children.as_slice() {
            if child.value == newchild.value {
                child.insert(newchild);
            }
        }
        self.children.push(child);
    }
    */
}


pub struct PositionSegment<'a> {
    path: &'static str,
    roots: Vec<PositionTrieNode<'a>>,
}

impl <'a> PositionSegment<'a> {
    pub fn calculate_position_tree_address(r12: u64, r34: u64, r56: u64, r78: u64) -> PositionTrieAddress {
        let first_address= ((r12 & 0x1111111100000000) >> 8) as u32;
        PositionTrieAddress {
            value: [
                ((r12 & 0xffffffff00000000) >> 48) as u16,
                ((r12 & 0x1111111100000000) >> 32) as u16,
                ((r12 & 0x1111111100000000) >> 16) as u16,
                r12 as u16,
                ((r34 & 0xffffffff00000000) >> 48) as u16,
                ((r34 & 0x1111111100000000) >> 32) as u16,
                ((r34 & 0x1111111100000000) >> 16) as u16,
                r34 as u16,
                ((r56 & 0xffffffff00000000) >> 48) as u16,
                ((r56 & 0x1111111100000000) >> 32) as u16,
                ((r56 & 0x1111111100000000) >> 16) as u16,
                r56 as u16,
                ((r78 & 0xffffffff00000000) >> 48) as u16,
                ((r78 & 0x1111111100000000) >> 32) as u16,
                ((r78 & 0x1111111100000000) >> 16) as u16,
                r78 as u16,
            ],
        }
    }

    /*
    pub fn fetch_child(&self, key: usize) -> Option<Arc<PositionTrieNode>> {
        self.roots[key]
    }
    */
    pub fn new() -> PositionSegment<'a> {
        PositionSegment {
            path: "segment.db",
            roots: Vec::new(),
        }
    }
}
