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
    pub value: [u16; 16]
}

pub struct PositionTrieConfig {
    children_size: usize,
}

pub struct PositionTrieNode {
    pub value: u16,
    pub children: Vec<Arc<PositionTrieNode>>, // size up to 2^16
    pub are_children_dirty: bool,
    pub is_terminal: bool,
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
    roots: Vec<PositionTrieNode>,
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

    fn insert_helper(&self, usize key, children: &mut Vec<Arc<PositionTrieNode>>) -> () {

    }

    pub fn insert(&self, address: PositionTrieAddress) -> () {

        let wanted_value = address.value[0];
        let mut found: bool = false;

        for root in self.roots {
            if root.value == wanted_value {
                self.insert_helper(, children)
                        // found match, merge our children
                        found = true;
                        let second_wanted_value = address.value[1];
                        let mut children = &root.children;
                        for wanted_value in address.value {
                            for child in children {
                                if child.value == wanted_value {
                                    // recurse..

                                }
                            }
                        };
                        let mut second_found: bool = false;
                        for child in root.children.iter() {
                            if child.value == second_wanted_value {
                                second_found = true;
                                for second_child in child.children.iter() {
                                    if second_child.value ==
                                }
                            };
                            if !second_found {
                                let node = PositionTrieNode::new(second_wanted_value, false);
                                root.children.push(Arc::new(node));
                            }

                        }
                        found = true;
                        ptn_arc.insert(childNode);
            }
        }
        if !found {
            // make new node, insert into roots
        };
    }

    /*
    pub fn fetch_child(&self, key: usize) -> Option<Arc<PositionTrieNode>> {
        self.roots[key]
    }
    */
    pub fn new() -> PositionSegment {
        PositionSegment {
            path: "segment.db",
            roots: Vec::new(),
        }
    }
}
