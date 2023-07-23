use std::net::TcpListener;

use goxoy_address_parser::address_parser::{ProtocolType, AddressParser, IPAddressVersion};

#[derive(Debug)]
pub struct SocketServer {
    pub started:bool,
    pub defined:bool,
    pub local_addr:String,
    //pub port_type:ProtocolType,
    //pub ip_address:IPAddressVersion
}

impl SocketServer {
    pub fn new() -> Self {
        SocketServer{
            local_addr:String::new(),
            started:false,
            defined:false
        }
    }
    pub fn new_with_config(protocol_type:ProtocolType, ip_address:String,port_no:usize,ip_version:IPAddressVersion) -> Self {
        let local_addr=AddressParser::object_to_string(AddressParser { 
            ip_address: ip_address, 
            port_no: port_no, 
            protocol_type: protocol_type, 
            ip_version: ip_version
        });
        SocketServer{
            local_addr:local_addr,
            defined:true,
            started:false,
        }
    }
    pub fn set_config(&mut self,config:AddressParser) {
        let local_addr=AddressParser::object_to_string(config);
        self.local_addr=local_addr;
        self.defined=true;
    }
    fn start_tcp(&mut self,addr_obj:AddressParser)->bool{
        dbg!(addr_obj.clone());
        let mut bind_str=addr_obj.ip_address;
        bind_str.push_str(":");
        bind_str.push_str(&addr_obj.port_no.to_string());
        let listener = TcpListener::bind(bind_str);
        if listener.is_err(){
            return false;
        }

        let listener=listener.unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            println!("Connection established!");
        }
        return true;
    }
    pub fn start(&mut self) -> bool{
        if self.defined==false{
            return false;
        }
        let addr_obj=AddressParser::string_to_object(self.local_addr.clone());
        if addr_obj.protocol_type==ProtocolType::TCP{
            self.start_tcp(addr_obj);
        }else{
            println!("start udp server");
            self.start_tcp(addr_obj);
        }
        return true;
    }
}

#[test]
fn full_test() {
    // cargo test  --lib full_test -- --nocapture
    let mut server_obj=SocketServer::new_with_config(
        ProtocolType::TCP,
        "127.0.0.1".to_string(), 
        1234, 
        IPAddressVersion::IpV4
    );
    println!("server_obj.local_addr: {}",server_obj.local_addr);
    server_obj.start();
    /*
    server_obj.local_addr
    let since_the_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    if since_the_epoch.is_ok(){
        let tmp_db_name=since_the_epoch.unwrap().as_micros().to_string();
        let mut kv_obj=KeyValueDb::new(&tmp_db_name);
        kv_obj.set_value("tmp_key", "tmp_value");
        let mut record_founded=false;
        let tmp_value=kv_obj.get_value("tmp_key");
        if tmp_value.is_some(){   
            if tmp_value.unwrap().eq("tmp_value"){
                record_founded=true;
            }
        }
        kv_obj.flush();
        if record_founded==true{
            assert!(true)
        }else{
            assert!(false)
        }
    }else{
        assert!(false)
    }
    */
    assert!(true)
}
