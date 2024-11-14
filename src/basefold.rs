use crate::field::FieldElement;
use crate::reed_muller::ReedMullerCode;
use rand::Rng;
use std::collections::HashMap;

pub struct BasefoldProtocol {
    pub code_family: Vec<ReedMullerCode>,
    pub t_vectors: Vec<Vec<FieldElement>>,
    commitment_randomness: Vec<FieldElement>,
    hash_table: HashMap<Vec<FieldElement>, Vec<FieldElement>>,
}

impl BasefoldProtocol {
    pub fn new(code_family: Vec<ReedMullerCode>, t_vectors: Vec<Vec<FieldElement>>) -> Self {
        let mut rng = rand::thread_rng();
        let commitment_randomness: Vec<FieldElement> = (0..code_family.len())
            .map(|_| FieldElement::new(rng.gen()))
            .collect();

        BasefoldProtocol {
            code_family,
            t_vectors,
            commitment_randomness,
            hash_table: HashMap::new(),
        }
    }

    pub fn commit(&self, message: &[FieldElement]) -> Vec<Vec<FieldElement>> {
        let mut oracles = Vec::new();
        let mut current = message.to_vec();
        oracles.push(current.clone());

        for (i, code) in self.code_family.iter().enumerate() {
            let encoded = code.encode(current.clone());
            let folded = self.fold_with_merkle(&encoded, &self.t_vectors[i], self.commitment_randomness[i]);
            current = folded;
            oracles.push(current.clone());
        }

        oracles
    }

    fn fold_with_merkle(
        &self,
        v: &[FieldElement],
        t: &[FieldElement],
        r: FieldElement,
    ) -> Vec<FieldElement> {
        let n = v.len();
        assert_eq!(n % 2, 0);
        let mut folded = Vec::with_capacity(n / 2);
        let mut merkle_tree = self.build_merkle_tree(v);

        for j in (0..n).step_by(2) {
            let t_j = t[j];
            let t_j1 = t[j + 1];
            let v_j = v[j];
            let v_j1 = v[j + 1];

            let numerator = v_j1 - v_j;
            let denominator = t_j1 - t_j;
            let slope = numerator * denominator.inverse();
            let value_at_r = slope * (r - t_j) + v_j;
            
            // Add Merkle proof
            let proof = self.generate_merkle_proof(&merkle_tree, j / 2);
            self.hash_table.insert(proof, vec![value_at_r]);
            
            folded.push(value_at_r);
        }

        folded
    }

    fn build_merkle_tree(&self, values: &[FieldElement]) -> Vec<Vec<FieldElement>> {
        let mut tree = Vec::new();
        let mut current_level = values.to_vec();
        tree.push(current_level.clone());

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                let hash = if chunk.len() == 2 {
                    self.hash_pair(&chunk[0], &chunk[1])
                } else {
                    chunk[0]
                };
                next_level.push(hash);
            }
            tree.push(next_level.clone());
            current_level = next_level;
        }

        tree
    }

    fn hash_pair(&self, left: &FieldElement, right: &FieldElement) -> FieldElement {
        // Pedersen commitment-based hashing
        let mut rng = rand::thread_rng();
        let r: u128 = rng.gen();
        let h = FieldElement::new(r);
        left * h + right
    }

    fn generate_merkle_proof(
        &self,
        tree: &[Vec<FieldElement>],
        index: usize,
    ) -> Vec<FieldElement> {
        let mut proof = Vec::new();
        let mut current_idx = index;

        for level in 0..tree.len() - 1 {
            let sibling_idx = if current_idx % 2 == 0 {
                current_idx + 1
            } else {
                current_idx - 1
            };

            if sibling_idx < tree[level].len() {
                proof.push(tree[level][sibling_idx]);
            }

            current_idx /= 2;
        }

        proof
    }

    pub fn verify_merkle_proof(
        &self,
        root: &FieldElement,
        value: &FieldElement,
        proof: &[FieldElement],
        index: usize,
    ) -> bool {
        let mut current = *value;
        let mut current_index = index;

        for sibling in proof {
            current = if current_index % 2 == 0 {
                self.hash_pair(&current, sibling)
            } else {
                self.hash_pair(sibling, &current)
            };
            current_index /= 2;
        }

        &current == root
    }

    pub fn query(&self, oracles: &[Vec<FieldElement>], lambda: usize) -> bool {
        let mut rng = rand::thread_rng();
        
        for _ in 0..lambda {
            let d = self.code_family.len() - 1;
            let mut mu = rng.gen_range(0..oracles[d].len());
            if mu % 2 != 0 {
                mu -= 1;
            }

            if !self.verify_query_path_with_merkle(oracles, mu) {
                return false;
            }
        }

        true
    }

    fn verify_query_path_with_merkle(&self, oracles: &[Vec<FieldElement>], mut mu: usize) -> bool {
        for i in (0..self.code_family.len()).rev() {
            let pi_i_plus1 = &oracles[i + 1];
            let pi_i = &oracles[i];
            let t_i = &self.t_vectors[i];
            let r = self.commitment_randomness[i];

            // Verify Merkle proof
            if let Some(proof) = self.hash_table.get(&pi_i_plus1[mu..mu + 2].to_vec()) {
                if !self.verify_merkle_proof(
                    &pi_i[mu / 2],
                    &proof[0],
                    &proof[1..],
                    mu / 2,
                ) {
                    return false;
                }
            }

            if !self.verify_fold_at_point(pi_i, pi_i_plus1, t_i, r, mu) {
                return false;
            }

            mu /= 2;
        }

        true
    }

    fn verify_fold_at_point(
        &self,
        pi_i: &[FieldElement],
        pi_i_plus1: &[FieldElement],
        t_i: &[FieldElement],
        r: FieldElement,
        mu: usize,
    ) -> bool {
        let v_mu = pi_i_plus1[mu];
        let v_mu_plus1 = pi_i_plus1[mu + 1];
        let t_mu = t_i[mu];
        let t_mu_plus1 = t_i[mu + 1];

        let numerator = v_mu_plus1 - v_mu;
        let denominator = t_mu_plus1 - t_mu;
        let slope = numerator * denominator.inverse();
        let expected = slope * (r - t_mu) + v_mu;

        pi_i[mu / 2] == expected
    }
}