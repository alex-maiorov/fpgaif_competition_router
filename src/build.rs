

fn main() {
    ::capnpc::CompilerCommand::new()
        .file("fpga-interchange-schema/interchange/DeviceResources.capnp")
        .import_path("src/")
        .run()
        .expect("compiling DeviceResources.capnp");

    ::capnpc::CompilerCommand::new()
        .file("fpga-interchange-schema/interchange/References.capnp")
        .import_path("src/")
        .run()
        .expect("compiling References.capnp");
    
    ::capnpc::CompilerCommand::new()
        .file("fpga-interchange-schema/interchange/LogicalNetlist.capnp")
        .import_path("src/")
        .run()
        .expect("compiling LogicalNetlist.capnp");

    ::capnpc::CompilerCommand::new()
        .file("fpga-interchange-schema/interchange/PhysicalNetlist.capnp")
        .import_path("src/")
        .run()
        .expect("compiling PhysicalNetlist.capnp");

    //panic!("done")
}