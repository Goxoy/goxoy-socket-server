# Goxoy Socket Server

Sunucu Soket kitaplığı. rust fonksiyonları kullanılarak geliştirilmiştir.

## Kullanım / Örnekler

```rust
    // doğrudan konfigüre ederek başlatabilirsiniz.
    let mut server_obj = SocketServer::new_with_config(
        ProtocolType::TCP,
        "127.0.0.1".to_string(),
        1234,
        IPAddressVersion::IpV4,
    );

```

  
## Lisans

[MIT](https://choosealicense.com/licenses/mit/)