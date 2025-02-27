use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated.rs");
    let mut f = File::create(&dest_path).unwrap();

    let runner = runner::Runner {};
    let operations = runner.list_operations();
    
    let mut generated_code = String::new();
    
    // Generate the trait definition
    generated_code.push_str("pub trait RunnerExtended {\n");
    for op in operations.iter() {
        let fn_name = format!("run_{}", op);
        let signature = format!(
            "    fn {}(&self, a: u32, b: u32) -> u32;\n",
            fn_name
        );
        generated_code.push_str(&signature);
    }
    generated_code.push_str("}\n\n");
    
    // Generate the implementation
    generated_code.push_str("impl RunnerExtended for crate::Runner {\n");
    for op in operations {
        let fn_name = format!("run_{}", op);
        let body = format!(
            r#"    fn {}(&self, a: u32, b: u32) -> u32 {{
        self.run_operation("{op}".to_string(), a, b)
    }}
"#,
            fn_name
        );
        generated_code.push_str(&body);
    }
    generated_code.push_str("}\n");

    f.write_all(generated_code.as_bytes()).unwrap();
    
    println!("cargo:rerun-if-changed=../runner/src/lib.rs");
    println!("cargo:warning=Generated code written to {:?}", dest_path);
}