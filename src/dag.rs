#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)] 

use core::hash;
use std::{collections::{HashMap, HashSet, VecDeque}, path::Ancestors, vec};

use crate::{types, Block, Hash, ValidatorId};

/*
    We implement DAG as an adjacency list with additional metadata on the round
    and the frontier

    should we make it pub?... leave it for now
*/
pub struct DAG {
    blocks: HashMap<Hash,Block>,
    
    //adj list
    children: HashMap<Hash, HashSet<Hash>>,
    parents: HashMap<Hash, HashSet<Hash>>,

    // quick lookup
    what_round: HashMap<u32, Vec<Hash>>,
    frontier: HashSet<Hash>,

    // curr round again
    curr_round: u32,
}

#[allow(clippy::derivable_impls)]
impl Default for DAG {
    fn default() -> Self {
        Self {
            blocks: HashMap::new(),
            children: HashMap::new(),
            parents: HashMap::new(),
            what_round: HashMap::new(),
            frontier: HashSet::new(),
            curr_round: 0,
        }
    }
}

impl DAG {
    pub fn insert_block(&mut self, block: Block) -> Result<(), String> {
        //let block_hash = block.hash;
        // since we clone we can still use it?
            // do we want to clone?...
        self.blocks.insert(block.hash, block.clone());
        // need to update children and parents and frontier
        // what about frontier removal?

        // add block to each parent's children list
        // add parents to this new block's parent list
        // remove parents from the frontier
        for parent_hash in &block.parents {
            self.children.entry(*parent_hash)
                // use or_insert_with for defautl value and returns mutable ref to the value
                .or_insert_with(HashSet::new) 
                // insert new HashSet or block.hash
                .insert(block.hash);

            self.parents.entry(block.hash)
                .or_default()
                .insert(*parent_hash);

            // note: frontier is the ones without children
            // so it's not just a visual thing of the latest era or the surface
            self.frontier.remove(parent_hash);
        }

        // new block is the frontier
        self.frontier.insert(block.hash);

        // update the round
        self.what_round.entry(block.round)
            .or_insert_with(Vec::new)
            .push(block.hash);

        Ok(())

    }

    // inside of a DAG tell me if the block exists
    pub fn get_block(&self, hash: &Hash) -> Option<&Block> {
        if self.blocks.contains_key(hash){
            return Some(self.blocks.get(hash).unwrap())
        }

        None

    }

    pub fn get_author_round_block(&self, author: ValidatorId, round: u32) -> Option<Hash> {
        if let Some(round_blocks) = self.what_round.get(&round) {
            for block_hash in round_blocks {
                if let Some(block) = self.blocks.get(block_hash) {
                    if block.author == author {
                        return Some(*block_hash);
                    }
                }
            }
        }

        None
    }

    // I guess if I don't need the actual Option Block
    pub fn contains_block(&self, hash: &Hash) -> bool {
        self.blocks.contains_key(hash)
    }

    pub fn get_frontier(&self) -> &HashSet<Hash> {
        &self.frontier
    }

}


#[test]
fn test_dag_methods() {
    let mut dummy_DAG = DAG::default();
    let dummy_block = Block::new(vec![], vec![], 1, 0);
    let hash = dummy_block.hash;

    let check = dummy_DAG.insert_block(dummy_block);

    assert_eq!(check, Ok(()));
    assert!(dummy_DAG.contains_block(&hash));
    assert!(dummy_DAG.get_block(&hash).is_some());

}

#[test]
fn test_dag_child_frontier() {
    let mut dummy_dag = DAG::default();
    let first = Block::new(vec![], vec![], 1, 0);
    let first_hash = first.hash;
    dummy_dag.insert_block(first).unwrap();

    // need to list parents - duh
    let second = Block::new(vec![], vec![first_hash], 1, 1);
    let second_hash = second.hash;
    dummy_dag.insert_block(second).unwrap();

    let third = Block::new(vec![], vec![], 2, 1);
    let third_hash = third.hash;
    dummy_dag.insert_block(third).unwrap();

    assert!(!dummy_dag.get_frontier().contains(&first_hash), "first failed");
    assert!(dummy_dag.get_frontier().contains(&second_hash), "second failed");
    assert!(dummy_dag.get_frontier().contains(&third_hash), "third failed");
    
}

impl DAG {
    pub fn get_parents(&self, hash: &Hash) -> Vec<Hash> {
        if let Some(set) = self.parents.get(hash) {
            let mut vec = Vec::new();
            for parent in set {

                // hash is [u8;32] which has copy trait so no clone
                // but vec is being dumb and needs actual val instead of ref
                // so use *parent which is direct value - compiler happy
                vec.push(*parent);
            }
            vec
        }
        else {
            // empty vec
            vec![]
        }
        
    }

    pub fn get_children(&self, hash: &Hash) -> Vec<Hash> {
        if let Some(set) = self.children.get(hash) {
            let mut vec = Vec::new();
            for child in set {
                // go through the hashmap and collect the children - why can't I just copy it
                vec.push(*child);
            }
            vec
        }
        else {
            // empty vec
            vec![]
        }
    }

    // run iterative DFS on DAG to get the ancestors
    pub fn get_ancestors(&self, hash: &Hash) -> HashSet<Hash> {
        let mut ancestors = HashSet::new();
        let mut stack = vec![*hash];

        while let Some(current) = stack.pop() {
            if let Some(parents) = self.parents.get(&current) {
                for parent in parents {
                    /*
                        if didn't exist in ancestors Hashset before
                        then this is new parent, and return true,
                        and is a valid ancestor

                        parents is &HashSet<Hash> so parent will be
                        &Hash. But .insert() needs owned Hash
                     */
                    if ancestors.insert(*parent) {
                        stack.push(*parent);
                    }
                }
            }
        }
        ancestors
    }

    pub fn get_descendants(&self, hash: &Hash) -> HashSet<Hash> {
        let mut descendants = HashSet::new();
        let mut stack = vec![*hash];

        while let Some(current) = stack.pop() {
            if let Some(_children) = self.children.get(&current) {
                for child in _children {

                    if descendants.insert(*child) {
                        stack.push(*child);
                    }
                }
            }
        }
        descendants
    }
}

#[test]
fn test_traversal() {
    let mut dummy_dag = DAG::default();
    let block_a = Block::new(vec![], vec![], 1, 0);
    let hash_a = block_a.hash;
    dummy_dag.insert_block(block_a).unwrap();

    let block_b = Block::new(vec![], vec![hash_a], 22, 1);
    let hash_b = block_b.hash;
    dummy_dag.insert_block(block_b).unwrap();

    let block_c = Block::new(vec![], vec![hash_a], 33, 1);
    let hash_c = block_c.hash;
    dummy_dag.insert_block(block_c).unwrap();

    let block_d = Block::new(vec![], vec![hash_b, hash_c], 44, 2);
    let hash_d = block_d.hash;
    dummy_dag.insert_block(block_d).unwrap();

    let block_e = Block::new(vec![], vec![hash_d], 55, 3);
    let hash_e = block_e.hash;
    dummy_dag.insert_block(block_e).unwrap();

    let old_blocks_e = dummy_dag.get_ancestors(&hash_e);
    let new_blocks_a = dummy_dag.get_descendants(&hash_a);

    // ah... im printing the hash...
    println!("ancestors for block e are: {:?}", old_blocks_e);
    println!("descendants for block a are {:?}", new_blocks_a);

    // e should have everything
    assert!(old_blocks_e.contains(&hash_a));
    assert!(old_blocks_e.contains(&hash_b));
    assert!(old_blocks_e.contains(&hash_c));
    assert!(old_blocks_e.contains(&hash_d));
    assert_eq!(old_blocks_e.len(), 4);

}

// TODO:
    // topological sort
    // path finding - lets say BFS
    // 
impl DAG {

    // run topo sort so we can get a path
    pub fn topological_sort(&self) -> Vec<Hash> {
        let mut result = Vec::new();
        let mut indegree: HashMap<Hash,usize> = HashMap::new();
        let mut queue = VecDeque::new();

        // what are my edges? in this case it's going to be children or parents
        for hash in self.blocks.keys() {
            let parent_count = self.parents.get(hash).map_or(0, |p| p.len());
            indegree.insert(*hash, parent_count);

            if parent_count == 0 {
                queue.push_back(*hash);
            }

        }

        while let Some(current) = queue.pop_front() {
            result.push(current);

            if let Some(children) = self.children.get(&current) {
                for child in children {
                    if let Some(degree) = indegree.get_mut(child) {
                        *degree -= 1;

                        // Topo sort - if we reach 0 degrees again then add it to the queue
                        if *degree == 0 {
                            queue.push_back(*child);
                        }
                    }
                }
            }
        }

        //let dummy_hash = [1u8;32];
        //return vec![dummy_hash]
        result
    }
    
    // use BFS - will give us the shortest path automatically
        // is that an issue?
    pub fn check_path(&self, from: &Hash, to: &Hash) -> Option<Vec<Hash>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        // base cases
            // if same destination - we just return the same thing
        if from == to {
            return Some(vec![*from]);
        }

        if !self.blocks.contains_key(from) {
            return None;
        }
        if !self.blocks.contains_key(to) {
            return None;
        }

        queue.push_back(*from);
        visited.insert(*from);

        // map to retrace
        let mut retrace_map: HashMap<Hash, Hash> = HashMap::new();

        while let Some(current) = queue.pop_front() {
            if current == *to {
                // construct the path back
                let mut path = Vec::new();
                let mut node = *to;

                while node != *from {
                    path.push(node);
                    node = retrace_map[&node];
                }
                path.push(*from);
                // when doing it via BFS we reverse it
                path.reverse();
                // returning Option
                return Some(path)
            }

            // else traverse
            if let Some(children) = self.children.get(&current) {
                for &child in children {
                    // for non-visited in children - add to queue
                    if !visited.contains(&child) {
                        visited.insert(child);
                        retrace_map.insert(child, current);
                        queue.push_back(child);
                    }
                }
            }
        }

        None
    }
}

#[test]
fn test_topo_path() {
    let mut dummy_dag = DAG::default();

    let block_a = Block::new(vec![], vec![], 1, 0);
    let hash_a = block_a.hash;
    dummy_dag.insert_block(block_a).unwrap();

    let block_b = Block::new(vec![], vec![hash_a], 22, 1);
    let hash_b = block_b.hash;
    dummy_dag.insert_block(block_b).unwrap();

    let block_c = Block::new(vec![], vec![hash_a], 33, 1);
    let hash_c = block_c.hash;
    dummy_dag.insert_block(block_c).unwrap();

    let block_d = Block::new(vec![], vec![hash_b, hash_c], 44, 2);
    let hash_d = block_d.hash;
    dummy_dag.insert_block(block_d).unwrap();

    let block_e = Block::new(vec![], vec![hash_d], 55, 3);
    let hash_e = block_e.hash;
    dummy_dag.insert_block(block_e).unwrap();

    let order = dummy_dag.topological_sort();
    //println!("The order is {:?}", order);
    for (i, hash) in order.iter().enumerate() {
        println!("  Step {}: [{:02x}{:02x}{:02x}{:02x}...]", i, hash[0], hash[1], hash[2], hash[3]);
    }

    println!("check path test");
    let path = dummy_dag.check_path(&hash_a, &hash_e);
    assert!(path.is_some());

    let contents = path.unwrap();
    for (i, hash) in contents.iter().enumerate() {
        println!("  Step {}: [{:02x}{:02x}{:02x}{:02x}...]", i, hash[0], hash[1], hash[2], hash[3]);
    }
}