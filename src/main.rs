use rand::Rng;
use crate::field::FieldElement;
use crate::reed_muller::ReedMullerCode;
use crate::basefold::BasefoldProtocol;

fn main() {
    // Initialize parameters
    let variables = 4;
    let degree = 2;
    let security_parameter = 40;
    
    // Create Reed-Muller code family
    let mut code_family = Vec::new();
    let mut t_vectors = Vec::new();
    
    for d in (1..=variables).rev() {
        let rm_code = ReedMullerCode::new(degree, d);
        code_family.push(rm_code);
        
        let n = 2_usize.pow(d as u32);
        let t_vector: Vec<FieldElement> = (0..n)
            .map(|i| FieldElement::new(i as u128))
            .collect();
        t_vectors.push(t_vector);
    }
    
    let protocol = BasefoldProtocol::new(code_family, t_vectors);
    
    // Test with random message
    let mut rng = rand::thread_rng();
    let message: Vec<FieldElement> = (0..protocol.code_family[0].k)
        .map(|_| FieldElement::new(rng.gen()))
        .collect();
    
    // Commit phase
    let oracles = protocol.commit(&message);
    
    // Verify phase
    let acceptance = protocol.query(&oracles, security_parameter);
    println!("Protocol verification result: {}", acceptance);
    
    // Test error detection
    let mut corrupted_oracles = oracles.clone();
    let random_index = rng.gen_range(0..corrupted_oracles[0].len());
    corrupted_oracles[0][random_index] = FieldElement::new(rng.gen());
    
    let rejection = protocol.query(&corrupted_oracles, security_parameter);
    println!("Corrupted oracle rejection: {}", !rejection);
}