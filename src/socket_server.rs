use goxoy_address_parser::address_parser::{AddressParser, IPAddressVersion, ProtocolType};
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
};

#[derive(Debug)]
pub struct SocketServer {
    started: bool,
    defined: bool,
    pub local_addr: String,
    callback: Option<fn(Vec<u8>)>,
    buffer_size: usize,
}

impl SocketServer {
    pub fn new() -> Self {
        SocketServer {
            local_addr: String::new(),
            started: false,
            defined: false,
            callback: None,
            buffer_size: 1024,
        }
    }
    pub fn new_with_config(
        protocol_type: ProtocolType,
        ip_address: String,
        port_no: usize,
        ip_version: IPAddressVersion,
    ) -> Self {
        let local_addr = AddressParser::object_to_string(AddressParser {
            ip_address: ip_address,
            port_no: port_no,
            protocol_type: protocol_type,
            ip_version: ip_version,
        });
        SocketServer {
            local_addr: local_addr,
            defined: true,
            started: false,
            callback: None,
            buffer_size: 1024,
        }
    }
    pub fn set_config(&mut self, config: AddressParser) {
        let local_addr = AddressParser::object_to_string(config);
        self.local_addr = local_addr;
        self.defined = true;
    }
    pub fn set_buffer_size(&mut self, buffer_size: usize) {
        self.buffer_size = buffer_size;
    }
    fn handle_client(&mut self, mut stream: TcpStream) {
        let mut data = [0 as u8; 1024];
        while match stream.read(&mut data) {
            Ok(size) => {
                if size > 0 {
                    println!("size: {}", size);
                    let written = stream.write(&data[0..size]);
                    if written.is_err() {
                        println!("data yazim hatasi");
                    } else {
                        //println!("data gonderildi");
                    }
                    self.callback.unwrap()(data.to_vec());
                }
                //stream.write(&data[0..size]).unwrap();
                true
            }
            Err(_) => {
                println!(
                    "An error occurred, terminating connection with {}",
                    stream.peer_addr().unwrap()
                );
                stream.shutdown(Shutdown::Both).unwrap();
                false
            }
        } {}
    }
    pub fn remove_assigned_callback(&mut self) {
        self.callback = None;
    }
    pub fn assign_callback(&mut self, callback: fn(Vec<u8>)) {
        self.callback = Some(callback);
    }
    fn start_tcp(&mut self, addr_obj: AddressParser) -> bool {
        if self.callback.is_none() {
            return false;
        }
        let mut bind_str = addr_obj.ip_address;
        bind_str.push_str(":");
        bind_str.push_str(&addr_obj.port_no.to_string());
        let listener = TcpListener::bind(bind_str);
        if listener.is_err() {
            return false;
        }

        let listener = listener.unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    self.handle_client(stream);
                    println!("Connection established!");
                }
                Err(_e) => {
                    println!("Connection error!");
                    /* connection failed */
                }
            }
        }
        return true;
    }
    pub fn start(&mut self) -> bool {
        if self.defined == false {
            return false;
        }
        self.started = true;
        let addr_obj = AddressParser::string_to_object(self.local_addr.clone());
        dbg!(addr_obj.clone());
        if addr_obj.protocol_type == ProtocolType::TCP {
            self.start_tcp(addr_obj);
        } else {
            println!("start udp server");
            self.start_tcp(addr_obj);
        }
        return true;
    }
}

#[test]
fn full_test() {
    // cargo test  --lib full_test -- --nocapture
    let mut server_obj = SocketServer::new_with_config(
        ProtocolType::TCP,
        "127.0.0.1".to_string(),
        1234,
        IPAddressVersion::IpV4,
    );
    println!("server_obj.local_addr: {}", server_obj.local_addr);
    server_obj.assign_callback(|data| {
        //println!("inside assign_callback");
        //dbg!(data.clone());
        let vec_to_string = String::from_utf8(data).unwrap(); // Converting to string
        println!("vec_to_string: {}", vec_to_string); // Output: Hello World
    });
    server_obj.start();
    assert!(true)
}
