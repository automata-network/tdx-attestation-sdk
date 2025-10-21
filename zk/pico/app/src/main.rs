#![no_main]

pico_sdk::entrypoint!(main);
use fibonacci_lib::{fibonacci, FibonacciData, serialize_fibonacci_data};
use pico_sdk::io::{commit_bytes, read_vec};

pub fn main() {
    // Read inputs `n` from the environment
    let input_bytes: Vec<u8> = read_vec();
    let n: u32 = u32::from_le_bytes(input_bytes[..4].try_into().unwrap());

    let a: u32 = 0;
    let b: u32 = 1;

    // Compute Fibonacci values starting from `a` and `b`
    let (a_result, b_result) = fibonacci(a, b, n);

    // Commit the assembled Fibonacci data as the public values in the Pico proof.
    // This allows the values to be verified by others.
    let result = FibonacciData {
        n,
        a: a_result,
        b: b_result,
    };
    let result_bytes = serialize_fibonacci_data(&result);

    commit_bytes(&result_bytes);
}
