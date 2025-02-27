fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build_deps/src/lib.rs");

    // Use the build_deps crate to get operations
    let operations = build_deps::fetch_operations();

    // Generate the operations module
    let mut content = String::new();
    content.push_str("// Auto-generated operations\n");

    for op in operations {
        content.push_str(&format!(
            r#"#[no_mangle]
pub extern "C" fn run_{op}(a: u32, b: u32) -> u32 {{
    run_operation("{op}".to_string(), a, b)
}}

"#
        ));
    }

    std::fs::write("src/generated_operations.rs", content).unwrap();
}