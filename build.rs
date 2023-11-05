use std::path::PathBuf;

fn main() {
    use rcgen::generate_simple_self_signed;
    let subject_alt_names = vec!["*.horcrust".to_string(), "localhost".to_string()];

    let cert = generate_simple_self_signed(subject_alt_names).unwrap();
    // The certificate is now valid for localhost and the domain "hello.world.example"

    // recreate the certificate only if it doesn't exists.
    if !PathBuf::from("./cert.pem").exists() {
        std::fs::write("./cert.pem", cert.serialize_pem().unwrap()).unwrap();
        std::fs::write("./key.pem", cert.serialize_private_key_pem()).unwrap();
    }
    prost_build::Config::new()
        .out_dir("src")
        .compile_protos(&["src/commands.proto"], &["src"])
        .expect("Could not compile protobuf types in commands.proto");
}
