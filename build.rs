// Build script for compiling Protocol Buffer definitions
// Only runs when websocket-streaming feature is enabled
//
// NOTE: Requires protoc (Protocol Buffer Compiler) to be installed:
// - Windows: choco install protoc or download from https://github.com/protocolbuffers/protobuf/releases
// - macOS: brew install protobuf
// - Linux: apt-get install protobuf-compiler or yum install protobuf-compiler

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "websocket-streaming")]
    {
        println!("cargo:rerun-if-changed=proto/yaticker.proto");
        
        // Try to compile protos, provide helpful error if protoc not found
        match prost_build::Config::new()
            .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
            .compile_protos(&["proto/yaticker.proto"], &["proto/"]) 
        {
            Ok(_) => {},
            Err(e) => {
                eprintln!("\n❌ Failed to compile Protocol Buffers!");
                eprintln!("   Error: {}", e);
                eprintln!("\n💡 You need to install protoc (Protocol Buffer Compiler):");
                eprintln!("   Windows: choco install protoc");
                eprintln!("           or download from https://github.com/protocolbuffers/protobuf/releases");
                eprintln!("   macOS:   brew install protobuf");
                eprintln!("   Linux:   apt-get install protobuf-compiler");
                eprintln!("            or yum install protobuf-compiler\n");
                return Err(Box::new(e));
            }
        }
    }
    
    Ok(())
}
