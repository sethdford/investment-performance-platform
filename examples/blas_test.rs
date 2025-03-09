use ndarray::{Array1, Array2};

fn main() {
    println!("Testing BLAS integration with ndarray");
    
    // Create two matrices
    let a = Array2::from_shape_vec((3, 2), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
    let b = Array2::from_shape_vec((2, 3), vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0]).unwrap();
    
    // Perform matrix multiplication (this will use BLAS if properly linked)
    let c = a.dot(&b);
    
    println!("Matrix A:\n{:?}", a);
    println!("Matrix B:\n{:?}", b);
    println!("Matrix C = A.dot(B):\n{:?}", c);
    
    // Create vectors for dot product
    let v1 = Array1::from_vec(vec![1.0, 2.0, 3.0]);
    let v2 = Array1::from_vec(vec![4.0, 5.0, 6.0]);
    
    // Calculate dot product (this will use BLAS if properly linked)
    let dot_product = v1.dot(&v2);
    
    println!("Vector v1: {:?}", v1);
    println!("Vector v2: {:?}", v2);
    println!("Dot product v1.dot(v2): {}", dot_product);
    
    println!("BLAS test completed successfully!");
} 