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



pub mod fpgaif{
    /*Notes:
    - The way they do referencing to list fields is a little cursed; an annotation is used to refer the index to the 
    original list field name. Not sure how to actually fish it out. Might hardcode it instead. 
    */

    //this is gonna be twice the system's address space width(i.e 16 bytes on a 64 bit system), 
    //methinks worth it to allow for indirection without having string-field name comparison madness

    use std::{fmt::Display, sync::{RwLock, Arc}};

    use capnp::serialize;

    use crate::serialization::physical_netlist_capnp::phys_netlist;

    struct VecIndexReference<T>{ 
        index: usize,
        vector_ref: Arc<RwLock<Vec<T>>>,
    }


    impl<T> VecIndexReference<T>{
        fn new(new_index: usize, new_vector_ref: Arc<RwLock<Vec<T>>>) -> VecIndexReference<T>{
            VecIndexReference{
                index: new_index,
                vector_ref: new_vector_ref,
            }
        }

        fn read_value(&self) -> Result<T>{
            return *(*(self.vector_ref).write())?[self.index];
        }

        fn write_value(&self, value: T)->Result<T>{
            *(*(self.vector_ref).write())?[self.index] = value;
        }
    }

    impl Display for VecIndexReference<T>{
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!("{}", self.read_value()?)
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
            let phys_netlist_message = message_reader.get_root::<phys_netlist::Reader>()?;

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




    struct PhysNetlist{
        part_name: String,
        placements: Vec<CellPlacement>,
        phys_cells: Vec<PhysCell>,
        str_list: Arc<RwLock<Vec<String>>,
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

    enum NetType{
        Signal,
        Gnd,
        Vcc,
    }

    impl PhysNet{
        
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

    impl RouteBranch{
        fn deserialize(str_list: Arc<RwLock<Vec<String>>>, reader: phys_netlist::route_branch::Reader) -> PhysBel {
            let route_segment: RouteSegment = match reader.which().unwrap(){
                phys_netlist::route_branch::Which::BelPin(b) => {
                    RouteSegment::BelPinSegment(PhysBelPin::deserialize(&str_list, b))
                },
                phys_netlist::route_branch::Which::SitePin(s) => {
                    RouteSegment::SitePinSegment(PhysBelPin::deserialize(&str_list, s))
                },
                phys_netlist::route_branch::Which::Pip(p) => {
                    RouteSegment::PipSegment(PhysPip::deserialize(&str_list, p))
                },
                phys_netlist::route_branch::Which::SitePIP(sp) => {
                    RouteSegment::SitePipSegment(PhysSitePip::deserialize(&str_list, sp))
                },
            };
            
            let mut branches: RouteBranch = Vec::new();
            if(reader.has_branches()){
                for branch in reader.get_branches().unwrap(){
                    branch_reader = branch.into_internal_struct_reader();
                    branches.push(RouteBranch::deserialize(str_list, branch_reader))
                }
            }
            RouteBranch{ 
                route_segment: route_segment, 
                branches:  branches
            }
        }
    }

    struct PhysBel{
        site: VecIndexReference<String>,
        bel: VecIndexReference<String>,
    }

    impl PhysBel{
        fn deserialize(str_list: Arc<RwLock<Vec<String>>>, reader: phys_netlist::phys_bel::Reader) -> PhysBel {
            PhysBel{
                site: VecIndexReference::new(reader.get_site(), str_list.clone()),
                bel: VecIndexReference::new(reader.get_bel(), str_list.clone()),
            }
        }
    }

    struct PhysBelPin{
        site: VecIndexReference<String>,
        bel: VecIndexReference<String>,
        pin: VecIndexReference<String>,
    }
    impl PhysBelPin{
        fn deserialize(str_list: Arc<RwLock<Vec<String>>>, reader: phys_netlist::phys_bel_pin::Reader){
            PhysBelPin{
                site: VecIndexReference::new(reader.get_site(), str_list.clone()),
                bel: VecIndexReference::new(reader.get_bel(), str_list.clone()),
                pin: VecIndexReference::new(reader.get_pin(), str_list.clone()),
            }
        }
    }

    struct PhysSitePin{
        site: VecIndexReference<String>,
        pin: VecIndexReference<String>,
    }

    impl PhysSitePin{
        fn deserialize(str_list: Arc<RwLock<Vec<String>>>, reader: phys_netlist::phys_site_pin::Reader){
            PhysSitePin {
                site: VecIndexReference::new(reader.get_site(), str_list.clone()),
                pin: VecIndexReference::new(reader.get_pin(), str_list.clone()),
            }
        }
    }


    struct PhysPip{
        tile: VecIndexReference<String>,
        wire0: VecIndexReference<String>,
        wire1: VecIndexReference<String>,
        forward: bool,
        is_fixed: bool,
        site: Option<VecIndexReference<String>>,
    }

    impl PhysPip{
        fn deserialize(str_list: Arc<RwLock<Vec<String>>>, reader: phys_netlist::phys_p_i_p::Reader)->PhysPip{
            let site: Option<VecIndexReference<String>>  = match reader.which().unwrap(){
                phys_netlist::phys_p_i_p::Which::NoSite(s) => None,
                phys_netlist::phys_p_i_p::Which::Site(s) => Some(VecIndexReference::new(s, str_list.clone())),
                _ => None, 
            };
            
            PhysPip{
                tile: VecIndexReference::new(reader.get_tile(), str_list.clone()),
                wire0: VecIndexReference::new(reader.get_wire0(), str_list.clone()),
                wire1: VecIndexReference::new(reader.get_wire1(), str_list.clone()),
                forward: reader.get_forward(),
                is_fixed: reader.get_is_fixed(),
                site: site,
            }
        }
    }


    
    struct PhysSitePip{
        site: VecIndexReference<String>,
        bel: VecIndexReference<String>,
        pin: VecIndexReference<String>,
        is_fixed: bool,
        inversion: Option<bool>, //None indicates inversion impossible, bool in option indicates if it is inverted currently
    }

    impl PhysSitePip{
        fn deserialize(str_list: Arc<RwLock<Vec<String>>>, reader: phys_netlist::phys_site_p_i_p::Reader)->PhysSitePip{
            
            let inversion_reader = reader.which().unwrap();
            let inversion: Option<bool> = match inversion_reader{
                phys_netlist::phys_netlist::phys_site_p_i_p::Which::IsInverting(b) => Some(b),
                phys_netlist::phys_netlist::phys_site_p_i_p::Which::Inverts => None,
            };
            
            PhysSitePip { 
                site: VecIndexReference::new(reader.get_site(), str_list.clone()), 
                bel: VecIndexReference::new(reader.get_bel(), str_list.clone()), 
                pin: VecIndexReference::new(reader.get_pin(), str_list.clone()), 
                is_fixed: VecIndexReference::new(reader.get_is_fixed(), str_list.clone()), 
                inversion: inversion }
        }
    }

    struct PhysNode{
        tile: VecIndexReference<String>,
        wire: VecIndexReference<String>,
        is_fixed: bool,
    }

    impl PhysNode{
        fn deserialize(str_list: Arc<RwLock<Vec<String>>>, reader: phys_netlist::phys_node::Reader)->PhysNode{
            PhysNode { 
                tile: VecIndexReference::new(reader.get_site(), str_list.clone()), 
                wire: VecIndexReference::new(reader.get_wire(), str_list.clone()), 
                is_fixed: reader.get_is_fixed() }
        }
    }


    struct SiteInstance{
        site: VecIndexReference<String>,
        site_inst_type: VecIndexReference<String>,
    }

    impl SiteInstance{
        fn deserialize(str_list: Arc<RwLock<Vec<String>>>, reader: phys_netlist::site_instance::Reader)->SiteInstance{
            SiteInstance{
                site: VecIndexReference::new(reader.get_site(), str_list.clone()),
                site_inst_type: VecIndexReference::new(reader.get_type(), str_list.clone()),
            }
        }
    }


    struct PhysNetlistProperty{
        key: VecIndexReference<String>,
        value: VecIndexReference<String>,
    }

    impl PhysNetlistProperty{
        fn deserialize(str_list: Arc<RwLock<Vec<String>>>, reader: phys_netlist::property::Reader)->PhysNetlistProperty{
            PhysNetlistProperty{
                key: VecIndexReference::new(reader.get_key(), str_list.clone()),
                value: VecIndexReference::new(reader.get_value(), str_list.clone()),
            }
        }
    }
//   struct PhysNode {
//     tile    @0 : StringIdx $stringRef();
//     wire    @1 : StringIdx $stringRef();
//     isFixed @2 : Bool;
//   } reader.get_value()?

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

