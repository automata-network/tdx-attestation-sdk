use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct FibonacciData {
    pub a: u32,
    pub b: u32,
    pub n: u32,
}

/// Computes the Fibonacci sequence starting from `a` and `b` up to the `n`-th iteration.
/// Returns the last two values in the sequence: (a, b).
pub fn fibonacci(mut a: u32, mut b: u32, n: u32) -> (u32, u32) {
    for _ in 0..n {
        let next = a.wrapping_add(b);
        a = b;
        b = next;
    }
    (a, b)
}

/// Loads an ELF file from the specified path.
pub fn load_elf(path: &str) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|err| {
        panic!("Failed to load ELF file from {}: {}", path, err);
    })
}

pub fn serialize_fibonacci_data(data: &FibonacciData) -> Vec<u8> {
    serde_json::to_vec(data).unwrap()
}

pub fn deserialize_fibonacci_data(data: &[u8]) -> FibonacciData {
    serde_json::from_slice(data).unwrap()
}
