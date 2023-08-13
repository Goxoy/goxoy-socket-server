# Goxoy Socket Server

Sunucu Soket kitaplığı. Rust fonksiyonları kullanılarak geliştirilmiştir.

## Kullanım / Örnekler

```rust
    // doğrudan konfigüre ederek başlatabilirsiniz.
    let mut server_obj = SocketServer::new_with_config(
        ProtocolType::TCP,
        "127.0.0.1".to_string(),
        1234,
        IPAddressVersion::IpV4,
    );

    //mesaj geldiğinde devreye girecek fonksiyon
    server_obj.on_receive(|sender_addr,income_data| {
        let vec_to_string = String::from_utf8(income_data.clone()).unwrap();
        println!("income callback => {} [ {} ]: {}", sender_addr, income_data.len(), vec_to_string);
    });

    // yeni kullanıcı bağlandığında devreye girecek fonksiyon
    server_obj.on_new_client(move |on_new_client| {
        println!("new client connected : {}", on_new_client);
    });

    // kullanıcının bağlantısı koptuğunda devreye girecek fonksiyon
    server_obj.on_client_disconnect(move |on_new_client| {
        println!("client disconnected : {}", on_new_client);
    });

    // işlem hatası olduğunda devreye girecek fonksiyon
    server_obj.on_error(|data| {
        println!("on error : {:?}", data);
    });

    //sunucuya başlatan komut
    server_obj.start();
    
    // istenen istemciye mesaj gönderecek fonksiyon
    server_obj.send(String::from("127.0.0.1:1234"), "welcome".as_bytes().to_vec());
    
```

  
## Lisans

[MIT](https://choosealicense.com/licenses/mit/)