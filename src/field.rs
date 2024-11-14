use std::ops::{Add, Sub, Mul, Div};
use std::fmt::{Debug, Display};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FieldElement {
    value: u128,
    modulus: u128,
}

#[derive(Error, Debug)]
pub enum FieldError {
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Invalid modulus")]
    InvalidModulus,
    #[error("Value exceeds modulus")]
    ValueExceedsModulus,
}

impl FieldElement {
    pub fn new(value: u128, modulus: u128) -> Result<Self, FieldError> {
        if modulus <= 1 {
            return Err(FieldError::InvalidModulus);
        }
        if value >= modulus {
            return Err(FieldError::ValueExceedsModulus);
        }
        Ok(Self { value, modulus })
    }

    pub fn zero(modulus: u128) -> Result<Self, FieldError> {
        Self::new(0, modulus)
    }

    pub fn one(modulus: u128) -> Result<Self, FieldError> {
        Self::new(1, modulus)
    }

    pub fn inverse(&self) -> Result<Self, FieldError> {
        if self.value == 0 {
            return Err(FieldError::DivisionByZero);
        }

        let (mut s, mut t, mut r) = (0i128, 1i128, self.modulus as i128);
        let (mut old_s, mut old_t, mut old_r) = (1i128, 0i128, self.value as i128);

        while r != 0 {
            let quotient = old_r / r;
            (old_r, r) = (r, old_r - quotient * r);
            (old_s, s) = (s, old_s - quotient * s);
            (old_t, t) = (t, old_t - quotient * t);
        }

        let mut result = old_s as u128;
        if result >= self.modulus {
            result %= self.modulus;
        }
        if old_s < 0 {
            result = self.modulus - ((-old_s as u128) % self.modulus);
        }
        
        Self::new(result, self.modulus)
    }

    pub fn pow(&self, mut exponent: u128) -> Self {
        let mut result = Self { value: 1, modulus: self.modulus };
        let mut base = *self;

        while exponent > 0 {
            if exponent & 1 == 1 {
                result = result * base;
            }
            base = base * base;
            exponent >>= 1;
        }
        result
    }

    pub fn sqrt(&self) -> Option<Self> {
        if self.value == 0 {
            return Some(*self);
        }

        if self.pow((self.modulus - 1) / 2).value != 1 {
            return None;
        }

        let mut q = self.modulus - 1;
        let mut s = 0;
        while q % 2 == 0 {
            q /= 2;
            s += 1;
        }

        let mut z = 2u128;
        while Self::new(z, self.modulus).unwrap().pow((self.modulus - 1) / 2).value == 1 {
            z += 1;
        }

        let mut m = s;
        let mut c = Self::new(z, self.modulus).unwrap().pow(q);
        let mut t = self.pow(q);
        let mut r = self.pow((q + 1) / 2);

        while t.value != 1 {
            let mut i = 0;
            let mut temp = t;
            while temp.value != 1 && i < m {
                temp = temp * temp;
                i += 1;
            }

            if i == 0 {
                return Some(r);
            }

            let b = c.pow(1u128 << (m - i - 1));
            m = i;
            c = b * b;
            t = t * c;
            r = r * b;
        }

        Some(r)
    }

    pub fn legendre_symbol(&self) -> i8 {
        let result = self.pow((self.modulus - 1) / 2).value;
        if result == 0 {
            0
        } else if result == 1 {
            1
        } else {
            -1
        }
    }
}

impl Add for FieldElement {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        assert_eq!(self.modulus, other.modulus, "Moduli must match");
        Self {
            value: (self.value + other.value) % self.modulus,
            modulus: self.modulus,
        }
    }
}

impl Sub for FieldElement {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        assert_eq!(self.modulus, other.modulus, "Moduli must match");
        Self {
            value: if self.value >= other.value {
                self.value - other.value
            } else {
                self.modulus - (other.value - self.value)
            },
            modulus: self.modulus,
        }
    }
}

impl Mul for FieldElement {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        assert_eq!(self.modulus, other.modulus, "Moduli must match");
        Self {
            value: ((self.value as u128) * (other.value as u128)) % self.modulus,
            modulus: self.modulus,
        }
    }
}

impl Div for FieldElement {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        assert_eq!(self.modulus, other.modulus, "Moduli must match");
        self * other.inverse().expect("Division by zero")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_field_arithmetic(a in 0u128..1000, b in 0u128..1000) {
            let modulus = 1009u128; // Prime modulus for testing
            if let (Ok(fa), Ok(fb)) = (FieldElement::new(a % modulus, modulus), FieldElement::new(b % modulus, modulus)) {
                let sum = fa + fb;
                let product = fa * fb;
                
                prop_assert!(sum.value < modulus);
                prop_assert!(product.value < modulus);
                
                if b % modulus != 0 {
                    let quotient = fa / fb;
                    prop_assert!(quotient.value < modulus);
                }
            }
        }
    }
}