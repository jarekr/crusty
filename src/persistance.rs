use std::{path::Path, sync::Arc};



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
pub struct PositionTrieAddress {
    value: [u16; 16]
}

pub struct PositionTrieConfig {
    children_size: usize,
}

pub struct PositionTrieNode {
    value: u16,
    children: Vec<Arc<PositionTrieNode>>, // size up to 2^16
    are_children_dirty: bool,
    is_terminal: bool,
}

impl PositionTrieNode {
    pub fn new(value: u16, is_terminal: bool) -> PositionTrieNode {
        PositionTrieNode {
            value: value,
            children: Vec::new(),
            are_children_dirty: false,
            is_terminal: is_terminal,
        }
    }

    pub fn insert(&self, child: PositionTrieNode) {
        self.children.push(Arc::new(child));
    }
}


pub struct PositionSegment {
    path: &'static str,
    roots: [Option<Arc<PositionTrieNode>>; 32]
}

impl PositionSegment {
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

    pub fn insert(&self, key: u16, childNode: PositionTrieNode) -> () {

        for root in self.roots {
            match root {
                Some(ptn_arc) => {
                    if ptn_arc.value == key {
                        // found match, merge our children
                        ptn_arc.insert(childNode);
                    }
                },
                None => _,

            }
        }

    }

    /*
    pub fn fetch_child(&self, key: usize) -> Option<Arc<PositionTrieNode>> {
        self.roots[key]
    }
    */
    pub fn new() -> PositionSegment {
        PositionSegment {
            path: "segment.db",
            roots: [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ]
        }
    }
}
