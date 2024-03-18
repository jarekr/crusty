use std::fs::{File, OpenOptions};
use std::io::Write;
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

pub struct Position {
 pub r12: u64,
 pub r34: u64,
 pub r56: u64,
 pub r78: u64,
}

impl Position {
    pub fn position_quad_to_bytes(&self) -> [u8; 32] {
        let mut result: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mut result_ptr = 0;
        for quad in [self.r12, self.r34, self.r56, self.r78] {
            for i in 0..3 {
                result[result_ptr + i] = (quad >> (248-i)) as u8;
            }
            result_ptr += 4;
        }
        result
    }
}
/*
impl PositionTrieAddress {
    pub fn to_bytes(&self) -> &[u8] {
        let mut return_bytes = u8kk
    }
}
*/

pub struct PositionTrieConfig {
    children_size: usize,
}

#[derive(Default, Debug)]
pub struct PositionTrieNode {
    pub value: u16,
    pub children: Vec<PositionTrieNode>, // size up to 2^16
    pub are_children_dirty: bool,
    pub is_terminal: bool,
}

impl PositionTrieNode {
    pub fn new(value: u16, terminal: bool) -> Self {
        PositionTrieNode {
            value: value,
            children: Vec::<PositionTrieNode>::new(),
            are_children_dirty: false,
            is_terminal: terminal,
        }
    }

    /*
    pub fn add_child(&mut self, child: RwLock<PositionTrieNode<'a>>) -> &'a mut PositionTrieNode {
        self.children.push(child);
        self
    }
    */

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /*
    pub fn get_node(&self, index: usize) -> Option<&RwLock<PositionTrieNode>> {
        self.children.get(index).or
    }

    pub fn get_mut_node(&mut self, index: usize) -> Option<&mut RwLock<PositionTrieNode>> {
        self.children.get_mut(index)
    }
    */

    // given a u16 value, find a PositionTrieNode with a matching value and return the node's index as OK(), or
    // return an ERR with the index that should have been there
    pub fn get_node_index_by_value(&self, value: u16) -> Result<usize, usize> {
        let mut index: usize = 0;
        for node in self.children.iter() {
            if node.value == value {
                return Ok(index);
            }
            index += 1;
        }
        Err(index)
    }
}

pub struct PositionTrie {
    root: PositionTrieNode,
}

impl PositionTrie {
    pub fn new() -> Self {
        PositionTrie {
            root: PositionTrieNode::default(),
        }
    }

    pub fn statt(&self) {
        let mut current_node = &self.root;
        let mut current_node_idx: usize = 0;
        let mut nodes  = Vec::<&PositionTrieNode>::new();

        let level_count = 16;

        nodes.push(current_node);

        for level in 0..(level_count)  {
            //let mut maybe_node: Option<&mut PositionTrieNode> = None;
            let mut child_count = 0;
            let mut new_nodes  = Vec::<&PositionTrieNode>::new();
            let nodes_len = nodes.len();
            for node in nodes {
                child_count += node.child_count();
                new_nodes.extend(node.children.iter());
            };
            println!("Level {}, node_count: {}, child_count: {}", level, nodes_len, child_count);
            nodes = new_nodes;
        }
    }


    pub fn insert(&mut self, pos: &PositionTrieAddress) -> i32 {
        let mut current_node = &mut self.root;
        let mut current_node_idx: usize = 0;

        let level_count = 16;
        let mut seen_levels = 0;

        for level in 0..(level_count - 1)  {
            //let mut maybe_node: Option<&mut PositionTrieNode> = None;
            current_node_idx = match current_node.get_node_index_by_value(pos.value[level]) {
                Ok(idx) => {
                    seen_levels += 1;
                    idx
                    //current_node.children.get(idx).as_mut().unwrap()
                }
                Err(idx2) => {
                    let newnode = PositionTrieNode::new(pos.value[level], level >= (level_count - 1));
                    current_node.children.push(newnode);
                    idx2
                    //current_node.children.get(idx2).as_mut().unwrap()
                }
            };
            current_node = current_node.children.get_mut(current_node_idx).unwrap();
        }
        seen_levels
    }

}


pub struct PositionSegment {
    path: &'static str,
    roots: Vec<Position>,
}

impl PositionSegment {
    pub fn new(path: &'static str) -> Self {
        PositionSegment {
            path: path,
            roots: Vec::<Position>::new(),
        }
    }

    pub fn get_header(&self) -> [u8; 8] {
        let fixed_header: [u8; 4] = [0xcc, 0xdd, 0x69, 0x42];

        let mut return_header: [u8; 8] = [0xcc, 0xdd, 0x69, 0x42, 0, 0, 0, 0];
        let len = self.roots.len() as u32;
        return_header[4] = (len >> 24) as u8;
        return_header[5] = (len >> 16) as u8;
        return_header[6] = (len >> 8) as u8;
        return_header[7] = len as u8;
        return_header
    }

    pub fn write(&self, path: &Path) -> Result<usize, std::io::Error> {
        let mut fh = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .expect("Unable to create or open for append position segment file");

        match fh.write_all(&self.get_header()) {
            Ok(_) => (),
            Err(e) => return Err(e),
        };

        for pos in self.roots.iter() {
            match fh.write_all(&pos.position_quad_to_bytes()) {
                Ok(_) => (),
                Err(err) => return Err(err),
            }
        };

        match fh.sync_all() {
            Ok(_) => Ok(self.roots.len()),
            Err(err) => Err(err)
        }
    }

    pub fn position_quad_to_bytes(r12: u64, r34: u64, r56: u64, r78: u64) -> [u8; 32] {
        let mut result: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mut result_ptr = 0;
        for quad in [r12, r34, r56, r78] {
            for i in 0..3 {
                result[result_ptr + i] = (quad >> (248-i)) as u8;
            }
            result_ptr += 4;
        }
        result
    }

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
}
