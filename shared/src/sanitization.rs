let xss_patterns = [
    "<script", "javascript:", "onerror=", "onload=", "eval(",
    "document.cookie", "document.write", "alert(", "prompt(",
    "confirm(", "iframe", "vbscript:", "data:", "base64",
];

let input_lower = input.to_lowercase();

for pattern in &xss_patterns {
    if input_lower.contains(&pattern.to_lowercase()) {
        warn!("Potential XSS attack detected: {}", input);
        return true;
    }
}

false
}

/// Check for path traversal
pub fn check_path_traversal(input: &str) -> bool {
    let traversal_patterns = [
        "../", "..\\", "%2e%2e%2f", "%2e%2e/", "..%2f",
        "%2e%2e%5c", "%2e%2e\\", "..%5c", "..%c0%af",
    ];
    
    for pattern in &traversal_patterns {
        if input.contains(pattern) {
            warn!("Potential path traversal attack detected: {}", input);
            return true;
        }
    }
    
    false
}

/// Sanitize input
pub fn sanitize_input(input: &str) -> Result<String, String> {
    // Check for malicious patterns
    if check_sql_injection(input) {
        return Err("Input contains potential SQL injection".to_string());
    }
    
    if check_xss(input) {
        return Err("Input contains potential XSS attack".to_string());
    }
    
    if check_path_traversal(input) {
        return Err("Input contains potential path traversal attack".to_string());
    }
    
    // Sanitize the input
    Ok(sanitize_string(input))
} 