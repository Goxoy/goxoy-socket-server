#![allow(warnings, unused)]
use std::{
    io::{ErrorKind, Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
};
use std::sync::mpsc::{Sender, Receiver};

use goxoy_address_parser::address_parser::{AddressParser, IPAddressVersion, ProtocolType};

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
pub struct SocketServerMessageList {
    peer_addr:String,
    data:Vec<u8>
}
#[derive( Debug)]
pub struct SocketServer {
    url: String,
    started: bool,
    defined: bool,
    tx:Option<Sender<Vec<u8>>>,
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
            local_addr: String::new(),
            tx:None,
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
            local_addr: local_addr,
            tx:None,
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
    pub fn send(&mut self,peer_addr:String, data: Vec<u8>)->bool{
        if self.tx.is_some(){
            self.tx.as_mut().unwrap().send(data);
            return true;
        }
        false
    }
    fn start_udp(&mut self) -> bool {
        true
    }
    fn start_tcp(&mut self) -> bool {
        let listener = TcpListener::bind(&self.url);
        if listener.is_err() {
            if self.fn_error.is_some() {
                let fn_error_obj = self.fn_error.unwrap();
                fn_error_obj(SocketServerErrorType::Connection);
            }
            return false;
        }

        let server = listener.unwrap();
        if server.set_nonblocking(true).is_err(){
            return false;
        }
        //let server=server.unwrap();

        //println!("buraya geldi");
        //std::process::exit(0);
    
    
        let mut clients = vec![];
        let buffer_size=self.buffer_size;
        
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        self.tx=Some(tx.clone());
        loop {
            if let Ok((mut socket, addr)) = server.accept() {
                println!("Client {} connected", addr);
    
                let tx = tx.clone();
                clients.push(socket.try_clone().expect("failed to clone client"));
                thread::spawn(move || loop {
                    let mut buff = vec![0; buffer_size];
    
                    match socket.read_exact(&mut buff) {
                        Ok(_) => {
                            let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                            let msg = String::from_utf8(msg).expect("invalid utf8 message");
    
                            println!("{} {:?}", addr, msg);
                            tx.send(msg).expect("failed to send message to rx");
                        }
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                        Err(_) => {
                            println!("Closing connection with: {}", addr);
                            break;
                        }
                    }
                    thread::sleep(::std::time::Duration::from_millis(100));
                });
            }
    
            if let Ok(msg) = rx.try_recv() {
                clients = clients
                    .into_iter()
                    .filter_map(|mut client| {
                        let mut buff = msg.clone().into_bytes();
                        buff.resize(buffer_size, 0);
    
                        client.write_all(&buff).map(|_| client).ok()
                    })
                    .collect::<Vec<_>>();
            }
            thread::sleep(::std::time::Duration::from_millis(100));
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
            if self.fn_receive_data.is_none() {
                println!("callback did not define");
            } else {
                self.start_udp();
            }
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

    server_obj.on_new_client(move |on_new_client| {
        println!("new client connected : {}", on_new_client);
    });
    server_obj.on_error(|data| {
        println!("on error : {:?}", data);
    });
    server_obj.start();
    server_obj.send(String::from("127.0.0.1:1234"), "welcome".as_bytes().to_vec());
    assert!(true)
}
