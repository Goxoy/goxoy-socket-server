use core::time;
use goxoy_address_parser::address_parser::{AddressParser, IPAddressVersion, ProtocolType};
use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    thread,
};

#[derive(Debug)]
pub struct SocketServer {
    stream_list: HashMap<String, TcpStream>,
    started: bool,
    defined: bool,
    pub local_addr: String,
    callback: Option<fn(Vec<u8>)>,
    buffer_size: usize,
}

impl SocketServer {
    pub fn new() -> Self {
        SocketServer {
            stream_list: HashMap::new(),
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
            stream_list: HashMap::new(),
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
    fn handle_client(&mut self, mut stream: TcpStream) -> bool {
        let mut data = [0 as u8; 50];
        match stream.read(&mut data) {
            Ok(size) => {
                if size > 0 {
                    println!("income size : {}", size);
                    //println!("income data : {:?}",data);
                    let written = stream.write(&data[0..size]);
                    if written.is_err() {
                        println!("data yazim hatasi");
                    } else {
                        println!("data gonderildi");
                        if self.callback.is_some() {
                            let callback_obj = self.callback.unwrap();
                            callback_obj(data.to_vec());
                        }
                        let reply_arr = "ok".as_bytes();
                        let result = stream.write_all(&reply_arr);
                        if result.is_err() {
                            println!("write err");
                        } else {
                            println!("write OK");
                        }
                    }
                    println!("return");
                    return true;
                }
            }
            Err(e) => {
                println!(
                    "An error occurred, terminating connection with {}",
                    stream.peer_addr().unwrap()
                );
                dbg!(e);
                println!("shotdown");
                stream.shutdown(Shutdown::Both).unwrap();
                //break;
            }
        }
        return false;
    }
    pub fn remove_assigned_callback(&mut self) {
        self.callback = None;
    }
    pub fn assign_callback(&mut self, callback: fn(Vec<u8>)) {
        self.callback = Some(callback);
    }
    fn handle_sender(mut stream: TcpStream) -> io::Result<()> {
        // Handle multiple access stream
        let mut buf = [0; 512];
        for _ in 0..1000 {
            // let the receiver get a message from a sender
            let bytes_read = stream.read(&mut buf)?;
            // sender stream in a mutable variable
            if bytes_read == 0 {
                return Ok(());
            }
            stream.write(&buf[..bytes_read])?;
            // Print acceptance message
            //read, print the message sent
            println!("from the sender:{}", String::from_utf8_lossy(&buf));
            // And you can sleep this connection with the connected sender
            thread::sleep(time::Duration::from_secs(1));
        }
        // success value
        Ok(())
    }
    fn start_tcp_old(&mut self, addr_obj: AddressParser) -> bool {
        if self.callback.is_none() {
            println!("callback did not define");
            return false;
        }
        let mut bind_str = addr_obj.ip_address;
        bind_str.push_str(":");
        bind_str.push_str(&addr_obj.port_no.to_string());
        let listener = TcpListener::bind(bind_str);
        if listener.is_err() {
            println!("listener did not bind");
            return false;
        }
        println!("listener started");
        let listener = listener.unwrap();
        loop {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        self.handle_client(stream);
                        println!("handled data");
                        /*
                        let peer_addr=income_stream.peer_addr().unwrap().to_string();
                        if self.stream_list.contains_key(&peer_addr)==false{
                            self.stream_list.insert(peer_addr.clone(), income_stream);
                        }
                        let stream=self.stream_list.get(&peer_addr);
                        if stream.is_some(){
                            println!("stream object success");
                            std::thread::spawn(move|| {
                                //handle_client(stream,count)
                                SocketServer::handle_client(TcpStream::try_clone(stream.unwrap()).unwrap());
                                //handle_client(TcpStream::try_clone(stream.unwrap()).unwrap());
                            });
                        }else{
                            println!("stream object error");
                        }
                        */
                    }
                    Err(_e) => {
                        println!("Connection error!");
                        /* connection failed */
                    }
                }
            }
        }
        return true;
    }
    fn start_tcp(&mut self, addr_obj: AddressParser) -> bool {
        if self.callback.is_none() {
            println!("callback did not define");
            return false;
        }
        let mut bind_str = addr_obj.ip_address;
        bind_str.push_str(":");
        bind_str.push_str(&addr_obj.port_no.to_string());
        let receiver_listener = TcpListener::bind(bind_str);
        if receiver_listener.is_err() {
            println!("listener did not bind");
            return false;
        }
        /*
        https://www.section.io/engineering-education/how-to-build-a-client-server-application-using-rust/
        https://gist.github.com/postmodern/0cf7cf8ec008c3713ef76cc6f4b3ffc1
        https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
        */
        println!("listener started");
        let receiver_listener = receiver_listener.unwrap();
        // Getting a handle of the underlying thread.
        let mut thread_vec: Vec<thread::JoinHandle<()>> = Vec::new();
        // listen to incoming connections messages and bind them to a sever socket address.
        for stream in receiver_listener.incoming() {
            let stream = stream.expect("failed");
            // let the receiver connect with the sender
            let handle = thread::spawn(move || {
                //receiver failed to read from the stream
                SocketServer::handle_sender(stream).unwrap_or_else(|error| eprintln!("{:?}", error))
            });

            // Push messages in the order they are sent
            thread_vec.push(handle);
        }
        println!("buraya geldi");
        for handle in thread_vec {
            // return each single value Output contained in the heap
            handle.join().unwrap();
        }
        println!("bitis");
        return true;
    }
    pub fn start(&mut self) -> bool {
        if self.defined == false {
            return false;
        }
        self.started = true;
        let addr_obj = AddressParser::string_to_object(self.local_addr.clone());
        if addr_obj.protocol_type == ProtocolType::TCP {
            println!("start TCP server");
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
    /*
    fn handle_client(mut stream: TcpStream,count:i32) {
        let mut data = [0 as u8; 50]; // using 50 byte buffer
        loop{ match stream.read(&mut data) {
            Ok(size) => {

                // echo everything!
                println!("size {}",size);
                println!("{} yazildi {:?}",count,data);
                stream.write(&data[0..size]).unwrap();
                true
            },
            Err(_) => {
                println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                stream.shutdown(Shutdown::Both).unwrap();
                false
            }
        }
    }
    let listener = TcpListener::bind("0.0.0.0:1234").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 1234");
    let mut count=1;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("{} New connection: {}", count, stream.peer_addr().unwrap());
                std::thread::spawn(move|| {
                    handle_client(stream,count)
                });
                count=count+1;
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
    */
    // cargo test  --lib full_test -- --nocapture
    let mut server_obj = SocketServer::new_with_config(
        ProtocolType::TCP,
        "127.0.0.1".to_string(),
        1234,
        IPAddressVersion::IpV4,
    );
    println!("server_obj.local_addr: {}", server_obj.local_addr);
    server_obj.assign_callback(|data| {
        let vec_to_string = String::from_utf8(data).unwrap(); // Converting to string
        println!("income callback: {}", vec_to_string); // Output: Hello World
    });
    server_obj.start();
    assert!(true)
}
