use std::fs::File;
use capnp::{serialize_packed, serialize};
use std::env;
use flate2::bufread;
use flate2::Compression;
use std::io::{BufReader, Read};
use petgraph::graph;
use std::fmt::{Debug, Display};

pub mod serialization{
    pub mod deviceresources_capnp {
        include!(concat!(env!("OUT_DIR"), "/fpga-interchange-schema/interchange/DeviceResources_capnp.rs"));
    }
    
    pub mod logical_netlist_capnp {
        include!(concat!(env!("OUT_DIR"), "/fpga-interchange-schema/interchange/LogicalNetlist_capnp.rs"));
    }
    
    pub mod physical_netlist_capnp {
        include!(concat!(env!("OUT_DIR"), "/fpga-interchange-schema/interchange/PhysicalNetlist_capnp.rs"));
    }
    
    pub mod references_capnp {
        include!(concat!(env!("OUT_DIR"), "/fpga-interchange-schema/interchange/References_capnp.rs"));
    }
}

use serialization::physical_netlist_capnp::phys_netlist::NetType;

pub mod fpgaif{
    /*Notes:
    - The way they do referencing to list fields is a little cursed; an annotation is used to refer the index to the 
    original list field name. Not sure how to actually fish it out. Might hardcode it instead. 
    */

    //this is gonna be twice the system's address space width(i.e 16 bytes on a 64 bit system), 
    //methinks worth it to allow for indirectio without having string-field name comparison madness

    use std::fmt::Display;

    use capnp::serialize;

    use super::serialization::physical_netlist_capnp;

    struct VecIndexReference<T>{ 
        index: usize,
        vector_ref: &Vec<T>,
    }


    impl<T> VecIndexReference<T>{
        fn new(new_index: usize, new_vector_ref: &Vec<T>) -> VecIndexReference<T>{
            VecIndexReference{
                index: new_index,
                vector_ref: new_vector_ref,
            }
        }

        fn read_value(&self) -> T{
            return *(self.vector_ref)[self.index];
        }

        fn write_value(&self, value: T){
            *(self.vector_ref)[self.index] = value;
        }

        fn new_append(&self, vector_ref: &mut Vec<T>, value: T) -> VecIndexReference<T>{
            *(self.vector_ref).push(value);
            let index = *(self.vector_ref).len() - 1; 
            return VecIndexReference::new(index, vector_ref);
        }
    }

    // struct Device{
    //     name: String,
    //     string_list: Vec<String>,
    //     site_type_list: Vec<SiteType>,
    //     tile_type_list: Vec<TileType>,
    //     tile_list: Vec<Tile>,
    //     wires: Vec<Wire>,
    //     nodes: Vec<Node>,
    //     //primitives_libs: Netlist, //implement this later if needed
    //     exception_map: Vec<PrimToMacroExpansion>,
    //     cell_bel_map: Vec<CellBelMapping>,
    //     cell_inversions: Vec<CellInversion>,
    //     packages: Vec<Package>,
    //     lut_definitions: LutDefinitions,
    //     parameter_definitions: ParameterDefinitions,
    //     wire_types: Vec<WireType>,
    //     pip_timings: Vec<PipTiming>,

    //     routing_graph: Graph<FpgaNode, FpgaEdge>, 
    // }

    impl TryFrom<File> for PhysNetlist{
        type Error;

        fn try_from(value: File) -> Result<Self, Self::Error> {
            let mut phys_netlist_gzreader = flate2::read::GzDecoder::new(value);
            let header = phys_netlist_gzreader.header();
            match header{
                Some(_) => (),
                None => return Err("Invalid GZip Header"),
            }
            let mut phys_netlist_uncompressed_data: Vec<u8> = Vec::new();
            let unzip_result = phys_netlist_gzreader.read_to_end(&mut phys_netlist_uncompressed_data);
            match unzip_result{
                Some(_) => (),
                None => return Err("Failed to unzip archive"),
            }
            return PhysNetlist::try_from(phys_netlist_uncompressed_data);
        }   
    }

    impl TryFrom<Vec<u8>> for PhysNetlist{
        type Error;

        fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
            let mut capnp_reader_options = capnp::message::ReaderOptions::new();
            capnp_reader_options.traversal_limit_in_words(None);
            let message_reader = serialize::read_message(
                &phys_netlist_uncompressed_data[..],
                capnp_reader_options,
            )?;
            let phys_netlist_message = message_reader.get_root::<serialization::physical_netlist_capnp::phys_netlist::Reader>()?;

            let string_list: Vec<String> = Vec::new();



            let phys_netlist_struct = PhysNetlist{ 
                part_name: todo!(), 
                placements: todo!(), 
                phys_cells: todo!(), 
                str_list: todo!(), 
                site_instances: todo!(), 
                properties: todo!(), 
                null_net: null_net
            };
                
        }
        
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if there is an issue with the deserialization downstream.
    fn try_read_physnet(reader: physicalnetlist_capnp::phys_netlist::phys_net::Reader,
        str_list: &Vec<String>) -> Result<PhysNet> {
            todo!();
    }


    #[derive(Debug)]
    struct PhysNetlist{
        part_name: String,
        placements: Vec<CellPlacement>,
        phys_cells: Vec<PhysCell>,
        str_list: Vec<String>,
        site_instances: Vec<SiteInstance>,
        properties: Vec<PhysNetlistProperty>,
        null_net: PhysNet,
    }

    struct PinMapping{
        cell_pin: VecIndexReference<String>,
        bel: VecIndexReference<String>,
        bel_pin: VecIndexReference<String>,
        is_fixed: bool,
        multi_cell_mapping: Option<MultiCellPinMapping>, //option instead of union with none
    }

    struct MultiCellPinMapping{
        multi_cell: VecIndexReference<String>,
        multi_type: VecIndexReference<String>,
    }

    struct CellPlacement{
        cell_name: VecIndexReference<String>,
        cell_type: VecIndexReference<String>,
        site: VecIndexReference<String>,
        bel: VecIndexReference<String>,
        pin_map: Vec<PinMapping>,
        other_bels: Vec<VecIndexReference<String>>,
        is_bel_fixed: bool,
        is_site_fixed: bool,
        alt_site_type: VecIndexReference<String>,
    }

    struct PhysCell{
        cell_name: VecIndexReference<String>,
        phys_type: PhysCellType
    }

    enum PhysCellType{
        Locked,
        Port,
        Gnd,
        Vcc,
    }

    struct PhysNet{
        name: VecIndexReference<String>,
        sources: Vec<RouteBranch>,
        stubs: Vec<RouteBranch>,
        phys_net_type: NetType,
    }



    struct RouteBranch{
        route_segment: RouteSegment,
        branches: Vec<RouteBranch>,
    }

    enum RouteSegment{
        BelPinSegment(PhysBelPin),
        SitePinSegment(PhysSitePin),
        PipSegment(PhysPip),
        SitePipSegment(PhysSitePip),
    }

    struct PhysBel{
        site: VecIndexReference<String>,
        bel: VecIndexReference<String>,
    }

    struct PhysBelPin{
        site: VecIndexReference<String>,
        bel: VecIndexReference<String>,
        pin: VecIndexReference<String>,
    }

    struct PhysSitePin{
        site: VecIndexReference<String>,
        pin: VecIndexReference<String>,
    }

    struct PhysPip{
        tile: VecIndexReference<String>,
        wire0: VecIndexReference<String>,
        wire1: VecIndexReference<String>,
        forward: bool,
        is_fixed: bool,
        site: Option<VecIndexReference<String>>,
    }

    struct PhysSitePip{
        site: VecIndexReference<String>,
        bel: VecIndexReference<String>,
        pin: VecIndexReference<String>,
        is_fixed: bool,
        inversion: Option<bool>, //None indicates inversion impossible, bool in option indicates if it is inverted currently
    }

    struct PhysNode{
        tile: VecIndexReference<String>,
        wire: VecIndexReference<String>,
        is_fixed: bool,
    }

    struct SiteInstance{
        site: VecIndexReference<String>,
        site_inst_type: VecIndexReference<String>,
    }

    struct PhysNetlistProperty{
        key: VecIndexReference<String>,
        value: VecIndexReference<String>,
    }
//   struct PhysNode {
//     tile    @0 : StringIdx $stringRef();
//     wire    @1 : StringIdx $stringRef();
//     isFixed @2 : Bool;
//   }

//   struct SiteInstance {
//     site  @0 : StringIdx $stringRef();
//     type  @1 : StringIdx $stringRef();
//   }

//   struct Property {
//     key   @0 : StringIdx $stringRef();
//     value @1 : StringIdx $stringRef();
//   }
// }


}

