#![warn(unused_imports)]
use goxoy_address_parser::address_parser::{AddressParser, IPAddressVersion, ProtocolType};
use std::io::Read;
use std::io::Write;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
};

use crate::ThreadPool;

#[derive(Debug)]
pub enum SocketServerErrorType {
    SocketStartingError,
    DataSendingError,
    Connection,
    Communication,
}
#[derive(Debug)]
pub enum SocketServerStatus {
    Connected,
    Disconnected,
}

#[derive(Clone, Debug)]
pub struct SocketServer {
    url: String,
    stream_list: HashMap<String, bool>,
    started: bool,
    defined: bool,
    pub local_addr: String,
    fn_receive_data: Option<fn(Vec<u8>)>,
    fn_new_client: Option<fn(String)>,
    fn_error: Option<fn(SocketServerErrorType)>,
    buffer_size: usize,
}

impl SocketServer {
    pub fn new() -> Self {
        SocketServer {
            url: String::new(),
            stream_list: HashMap::new(),
            local_addr: String::new(),
            started: false,
            defined: false,
            fn_receive_data: None,
            fn_new_client: None,
            fn_error: None,
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
            url: String::new(),
            stream_list: HashMap::new(),
            local_addr: local_addr,
            defined: true,
            started: false,
            fn_receive_data: None,
            fn_new_client: None,
            fn_error: None,
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
    pub fn on_receive(&mut self, on_receive_data: fn(Vec<u8>)) {
        self.fn_receive_data = Some(on_receive_data);
    }
    pub fn on_new_client(&mut self, on_new_client: fn(String)) {
        self.fn_new_client = Some(on_new_client);
    }
    pub fn on_error(&mut self, on_error: fn(SocketServerErrorType)) {
        self.fn_error = Some(on_error);
    }
    pub fn handle_connection(
        mut stream: TcpStream,
        receive_data_func: Option<fn(Vec<u8>)>,
        error_func: Option<fn(SocketServerErrorType)>,
        new_client_func: Option<fn(String)>,
    ) {
        if new_client_func.is_some() {
            let peer_addr = stream.peer_addr().unwrap().to_string();
            new_client_func.unwrap()(peer_addr);
        }
        let mut buffer = [0; 4096];
        loop {
            let received_size = stream.read(&mut buffer);
            if received_size.is_ok() {
                let received_size = received_size.unwrap();
                if received_size > 0 {
                    if receive_data_func.is_some() {
                        receive_data_func.unwrap()(buffer[..received_size].to_vec());
                    }
                }
            } else {
                if error_func.is_some() {
                    error_func.unwrap()(SocketServerErrorType::Communication);
                }
                break;
            }
        }

        //stream.write("ok".as_bytes()).unwrap();
        //stream.flush().unwrap();
    }
    /*
    pub fn send(&mut self, data: Vec<u8>) -> bool {
        let write_result = stream.write(data.as_slice());
        if write_result.is_ok() {
    }
    */
    // https://www.youtube.com/watch?v=hzSsOV2F7-s
    fn start_tcp(&mut self) -> bool {
        let listener = TcpListener::bind(&self.url);
        if listener.is_err() {
            if self.fn_error.is_some() {
                let fn_error_obj = self.fn_error.unwrap();
                fn_error_obj(SocketServerErrorType::Connection);
            }
            return false;
        }

        let pool = ThreadPool::new(50);
        let listener = listener.unwrap();
        for stream in listener.incoming() {
            let received_cloned = self.fn_receive_data;
            let error_cloned = self.fn_error;
            let new_client_cloned = self.fn_new_client;
            pool.execute(move || {
                SocketServer::handle_connection(
                    stream.unwrap(),
                    received_cloned,
                    error_cloned,
                    new_client_cloned,
                );
            });
        }
        return true;
    }
    pub fn start(&mut self) -> bool {
        if self.defined == false {
            return false;
        }
        self.started = true;
        let addr_obj = AddressParser::string_to_object(self.local_addr.clone());
        if addr_obj.protocol_type == ProtocolType::TCP {
            //println!("start TCP server");
            let mut bind_str = addr_obj.ip_address;
            bind_str.push_str(":");
            bind_str.push_str(&addr_obj.port_no.to_string());
            self.url = bind_str.clone();
            if self.fn_receive_data.is_none() {
                println!("callback did not define");
            } else {
                self.start_tcp();
            }
        } else {
            //println!("start udp server");
            self.start_tcp();
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
    server_obj.on_receive(|data| {
        let vec_to_string = String::from_utf8(data.clone()).unwrap(); // Converting to string
        println!("income callback [ {} ]: {}", data.len(), vec_to_string); // Output: Hello World
    });
    server_obj.on_new_client(|on_new_client| {
        println!("new client connected : {}", on_new_client);
    });
    server_obj.on_error(|data| {
        println!("on error : {:?}", data);
    });
    server_obj.start();
    assert!(true)
}
