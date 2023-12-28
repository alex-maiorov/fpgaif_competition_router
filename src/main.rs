use std::fs::File;
use capnp::{serialize_packed, serialize};
use std::env;
use flate2::bufread;
use flate2::Compression;
use std::io::{BufReader, Read};
use petgraph::graph;

pub mod DeviceResources_capnp {
    include!(concat!(env!("OUT_DIR"), "/fpga-interchange-schema/interchange/DeviceResources_capnp.rs"));
}

pub mod LogicalNetlist_capnp {
    include!(concat!(env!("OUT_DIR"), "/fpga-interchange-schema/interchange/LogicalNetlist_capnp.rs"));
}

pub mod PhysicalNetlist_capnp {
    include!(concat!(env!("OUT_DIR"), "/fpga-interchange-schema/interchange/PhysicalNetlist_capnp.rs"));
}

pub mod References_capnp {
    include!(concat!(env!("OUT_DIR"), "/fpga-interchange-schema/interchange/References_capnp.rs"));
}

use crate::PhysicalNetlist_capnp::phys_netlist;
use crate::LogicalNetlist_capnp::netlist ;
use crate::DeviceResources_capnp::{device, string_ref, hash_set};

//note to V, code is poorly organized but will refactor later



fn main() {
    //use a la [something.phys] [something.device]
    println!("Usage: ./wip-router /path/to/design-netlist.phys /path/to/deviceresources.device");
    let args: Vec<String> = env::args().collect();

    let phys_netlist_file = File::open(&args[1]).expect("Unable to open file");
    let device_resources_file = File::open(&args[2]).expect("Unable to open file");
    
    //let compressed_data = std::fs::read(&args[1]).unwrap();

    /* begin lines taken from flate2-rs example */ 

    let mut phys_netlist_gzreader = flate2::read::GzDecoder::new(phys_netlist_file);
    let mut device_resources_gzreader = flate2::read::GzDecoder::new(device_resources_file);
    /* end lines taken from flate2-rs example */ 

    
    
    phys_netlist_gzreader.header().expect("Invalid Header in first argument(PhysicalNetlist)");
    device_resources_gzreader.header().expect("Invalid Header in second argument(DeviceResources)");
    
    let mut phys_netlist_uncompressed_data = Vec::new();
    phys_netlist_gzreader.read_to_end(&mut phys_netlist_uncompressed_data).unwrap();

    println!("Unzipped physical netlist containing {} bytes of uncompressed data", phys_netlist_uncompressed_data.len());


    let mut device_resources_uncompressed_data = Vec::new();
    device_resources_gzreader.read_to_end(&mut device_resources_uncompressed_data).unwrap();

    println!("Unzipped device resources containing {} bytes of uncompressed data", device_resources_uncompressed_data.len());
    //highly cursed and will use a ton of RAM, errors out otherwise.
    let mut capnp_reader_options = capnp::message::ReaderOptions::new();
    capnp_reader_options.traversal_limit_in_words(None);
    /* begin Lines taken from capnproto-rust example */ 
    let message_reader = serialize::read_message(
        &phys_netlist_uncompressed_data[..],
        capnp_reader_options,
    ).unwrap();
    
    let phys_netlist_message = message_reader.get_root::<PhysicalNetlist_capnp::phys_netlist::Reader>().unwrap();
    /* end Lines taken from capnproto-rust example */ 

    let part_name = phys_netlist_message.get_part().unwrap().to_str().unwrap();
    println!("Part Name in PhysicalNetlist: {}", part_name);

    
    

    let device_message_reader = serialize::read_message(
        &device_resources_uncompressed_data[..],
        capnp_reader_options,
    ).unwrap();

    let device_message = message_reader.get_root::<DeviceResources_capnp::device::Reader>().unwrap();
    let device_name = device_message.get_name().unwrap().to_str().unwrap();
    println!("Part Name in DeviceResources: {}", device_name);

    
    //println!("{}", design_name);
    
    //let mut device_routing_graph: graph::Graph<some_node_struct, some_edge_struct> = graph::Graph::new();

    //obv replace some_xxxx_struct with actual structs

    //above will be populated by the deviceresources and phys netlist as in poc router

    // for tile in device_message.get_tile_list().unwrap(){
    //     let name_strref = tile.get_name();
    //     let name = string_ref::get_type().which().into();
    //     println!("Tile Name: {}", name);
    // }

    return;
    //println!(PhysicalNetlist_capnp::phys_netlist::get_name());
    //println!("Hello, world!");

    //use the program as "cargo run [gzipped file]"
}
