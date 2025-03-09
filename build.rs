fn main() {
    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search=/opt/homebrew/opt/openblas/lib");
    
    // Tell cargo to tell rustc to link the system openblas shared library
    println!("cargo:rustc-link-lib=openblas");
    
    // Only re-run the build script when build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
} 