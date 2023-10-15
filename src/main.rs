/* #[inline(never)]
fn calc_histogram(histogram: &mut [u32], records: &[u8], nb_records: usize) {
    for i in 0..nb_records {
        unsafe {
            *histogram.get_unchecked_mut(*records.get_unchecked(i) as usize) += 1;
        }
    }
}

fn main() {
    let mut histogram = vec![0u32; 256]; // Initialize a histogram with zeros
    let records: Vec<u8> = vec![1, 2, 3, 4];

    calc_histogram(&mut histogram, &records, records.len());

    for (value, count) in histogram.iter().enumerate() {
        if *count > 0 {
            println!("Value {}: Count {}", value, count);
        }
    }
} */

fn main() {
    let mut result = vec![0; 15]; // Creating a mutable vector of integers with initial values set to 0.
    let input_a = vec![1, 2, 3, 4, 51, 2, 3, 4, 51, 2, 3, 4, 51, 2, 3];
    let input_b = vec![5, 4, 3, 2, 15, 4, 3, 2, 15, 4, 3, 2, 15, 4, 1];

    atoi_simd::example_for(&mut result, &input_a, &input_b, 5);
    // example_for(&mut result, &input_a, &input_b, 15);
    // example_for(&mut result, &input_a, &input_b, 10);

    println!("Result: {:?}", result);
}
