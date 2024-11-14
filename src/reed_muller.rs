use crate::field::FieldElement;
use std::collections::HashMap;

pub struct ReedMullerCode {
    pub degree: usize,
    pub variables: usize,
    pub n: usize,
    pub k: usize,
    pub generator_matrix: Vec<Vec<FieldElement>>,
    pub parity_check_matrix: Vec<Vec<FieldElement>>,
    evaluation_points: Vec<Vec<u8>>,
    weight_enumerator: HashMap<usize, usize>,
}

impl ReedMullerCode {
    pub fn new(degree: usize, variables: usize) -> Self {
        let n = 2_usize.pow(variables as u32);
        let k = Self::compute_dimension(degree, variables);
        let evaluation_points = Self::generate_evaluation_points(variables);
        let generator_matrix = Self::generate_generator_matrix(degree, variables, &evaluation_points);
        let parity_check_matrix = Self::generate_parity_check_matrix(degree, variables, &evaluation_points);
        let weight_enumerator = Self::compute_weight_enumerator(degree, variables);

        ReedMullerCode {
            degree,
            variables,
            n,
            k,
            generator_matrix,
            parity_check_matrix,
            evaluation_points,
            weight_enumerator,
        }
    }

    pub fn encode(&self, message: Vec<FieldElement>) -> Vec<FieldElement> {
        assert_eq!(message.len(), self.k);
        let mut codeword = vec![FieldElement::zero(); self.n];
        
        for i in 0..self.k {
            for j in 0..self.n {
                codeword[j] = codeword[j] + message[i] * self.generator_matrix[i][j];
            }
        }
        
        codeword
    }

    pub fn decode(&self, received: Vec<FieldElement>) -> Vec<FieldElement> {
        assert_eq!(received.len(), self.n);
        let mut decoded = vec![FieldElement::zero(); self.k];
        
        // Majority logic decoding for Reed-Muller codes
        for i in (0..=self.degree).rev() {
            let subspaces = self.generate_subspaces(i);
            for j in 0..self.k {
                let mut votes = 0i32;
                for subspace in &subspaces {
                    let eval = self.evaluate_on_subspace(&received, subspace);
                    if eval.value > FIELD_SIZE / 2 {
                        votes += 1;
                    } else {
                        votes -= 1;
                    }
                }
                decoded[j] = if votes > 0 { FieldElement::one() } else { FieldElement::zero() };
            }
        }
        
        decoded
    }

    fn generate_subspaces(&self, dimension: usize) -> Vec<Vec<Vec<u8>>> {
        let mut subspaces = Vec::new();
        let mut basis = Vec::new();
        self.generate_basis_vectors(dimension, &mut basis, &mut subspaces);
        subspaces
    }

    fn generate_basis_vectors(
        &self,
        remaining_dim: usize,
        current_basis: &mut Vec<Vec<u8>>,
        subspaces: &mut Vec<Vec<Vec<u8>>>
    ) {
        if current_basis.len() == remaining_dim {
            subspaces.push(self.generate_subspace_from_basis(current_basis));
            return;
        }

        let start = if current_basis.is_empty() { 0 } else { current_basis.last().unwrap()[0] as usize + 1 };
        for i in start..self.variables {
            let mut new_vector = vec![0; self.variables];
            new_vector[i] = 1;
            current_basis.push(new_vector);
            self.generate_basis_vectors(remaining_dim, current_basis, subspaces);
            current_basis.pop();
        }
    }

    fn generate_subspace_from_basis(&self, basis: &[Vec<u8>]) -> Vec<Vec<u8>> {
        let mut subspace = vec![vec![0; self.variables]];
        for i in 0..(1 << basis.len()) {
            let mut point = vec![0; self.variables];
            for j in 0..basis.len() {
                if (i >> j) & 1 == 1 {
                    for k in 0..self.variables {
                        point[k] ^= basis[j][k];
                    }
                }
            }
            subspace.push(point);
        }
        subspace
    }

    fn evaluate_on_subspace(&self, received: &[FieldElement], subspace: &[Vec<u8>]) -> FieldElement {
        let mut sum = FieldElement::zero();
        for point in subspace {
            let idx = self.point_to_index(point);
            sum = sum + received[idx];
        }
        sum
    }

    fn point_to_index(&self, point: &[u8]) -> usize {
        let mut idx = 0;
        for (i, &bit) in point.iter().enumerate() {
            if bit == 1 {
                idx |= 1 << i;
            }
        }
        idx
    }

    fn generate_parity_check_matrix(
        degree: usize,
        variables: usize,
        evaluation_points: &[Vec<u8>],
    ) -> Vec<Vec<FieldElement>> {
        let n = 2_usize.pow(variables as u32);
        let k = Self::compute_dimension(degree, variables);
        let r = n - k;
        let mut matrix = Vec::with_capacity(r);

        // Generate dual space basis
        let mut dual_basis = Vec::new();
        for d in (degree + 1)..=variables {
            for combination in Self::generate_combinations(variables, d) {
                let mut row = vec![FieldElement::zero(); n];
                for (j, point) in evaluation_points.iter().enumerate() {
                    let mut eval = FieldElement::one();
                    for &var in &combination {
                        if point[var] == 1 {
                            eval = eval * FieldElement::new(1);
                        }
                    }
                    row[j] = eval;
                }
                matrix.push(row);
            }
        }

        matrix
    }

    fn generate_combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
        let mut result = Vec::new();
        let mut combination = Vec::new();
        Self::combinations_helper(n, k, 0, &mut combination, &mut result);
        result
    }

    fn combinations_helper(
        n: usize,
        k: usize,
        start: usize,
        combination: &mut Vec<usize>,
        result: &mut Vec<Vec<usize>>,
    ) {
        if combination.len() == k {
            result.push(combination.clone());
            return;
        }

        for i in start..n {
            combination.push(i);
            Self::combinations_helper(n, k, i + 1, combination, result);
            combination.pop();
        }
    }
}