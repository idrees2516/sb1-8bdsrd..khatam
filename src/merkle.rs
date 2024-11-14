use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use crate::field::FieldElement;
use crate::error::CryptoError;

pub struct MerkleTree {
    layers: Vec<Vec<[u8; 32]>>,
    leaves: Vec<[u8; 32]>,
    leaf_values: HashMap<[u8; 32], Vec<u8>>,
}

impl MerkleTree {
    pub fn new<T: AsRef<[u8]>>(values: &[T]) -> Self {
        let mut hasher = Sha3_256::new();
        let mut leaves = Vec::with_capacity(values.len());
        let mut leaf_values = HashMap::new();
        
        for value in values {
            hasher.update(value);
            let mut leaf = [0u8; 32];
            leaf.copy_from_slice(&hasher.finalize_reset());
            leaf_values.insert(leaf, value.as_ref().to_vec());
            leaves.push(leaf);
        }
        
        let mut layers = vec![leaves.clone()];
        let mut current_layer = leaves;
        
        while current_layer.len() > 1 {
            let mut next_layer = Vec::new();
            for chunk in current_layer.chunks(2) {
                let mut hasher = Sha3_256::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]);
                }
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&hasher.finalize());
                next_layer.push(hash);
            }
            layers.push(next_layer.clone());
            current_layer = next_layer;
        }
        
        Self {
            layers,
            leaves: layers[0].clone(),
            leaf_values,
        }
    }
    
    pub fn root(&self) -> [u8; 32] {
        self.layers.last().unwrap()[0]
    }
    
    pub fn generate_proof(&self, index: usize) -> Result<MerkleProof, CryptoError> {
        if index >= self.leaves.len() {
            return Err(CryptoError::InvalidParameters("Index out of bounds".into()));
        }
        
        let mut proof = Vec::new();
        let mut current_idx = index;
        
        for layer in &self.layers[..self.layers.len() - 1] {
            let sibling_idx = if current_idx % 2 == 0 {
                current_idx + 1
            } else {
                current_idx - 1
            };
            
            if sibling_idx < layer.len() {
                proof.push((layer[sibling_idx], current_idx % 2 == 0));
            }
            
            current_idx /= 2;
        }
        
        Ok(MerkleProof {
            proof,
            leaf: self.leaves[index],
            index,
        })
    }
}

pub struct MerkleProof {
    proof: Vec<([u8; 32], bool)>,
    leaf: [u8; 32],
    index: usize,
}

impl MerkleProof {
    pub fn verify(&self, root: [u8; 32]) -> bool {
        let mut current_hash = self.leaf;
        let mut hasher = Sha3_256::new();
        
        for &(ref sibling, is_right) in &self.proof {
            hasher.reset();
            if is_right {
                hasher.update(&current_hash);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(&current_hash);
            }
            current_hash.copy_from_slice(&hasher.finalize());
        }
        
        current_hash == root
    }
}