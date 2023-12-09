use async_std::net::{UdpSocket, SocketAddrV4};
use std::error::Error;
use std::net::Ipv4Addr;
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::net:: SocketAddr;
use std::time::{Duration, Instant};
use std::u128::MAX;
extern crate image;
use image::{DynamicImage, ImageBuffer, Rgba};
use image::GenericImageView;
use image::{ ImageOutputFormat};
use std::io::Cursor;
use base64;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use steganography::util::*;
use steganography::decoder::*;
use lazy_static::lazy_static;
use std::io;
use tokio::time::sleep;
use std::io::prelude::*;
use tokio::time::timeout;
use show_image::{create_window, ImageInfo, ImageView};
// use minifb::{Key, Window, WindowOptions};


struct Client {
    name: String,
    ip_address: String
}
struct requestingClient {
    image_id: String,
    requesting_client_name: String,
    access_rights: String,

}

lazy_static! {
    static ref REQUESTING_CLIENTS: Mutex<Vec<requestingClient>> = Mutex::new(Vec::new());
}
struct requestedImages{
    image_id: String,
    sender_client_name: String,
    access_rights: String,
    image_data: Vec<u8>,
}
lazy_static! {
    static ref REQUESTED_IMAGES: Mutex<Vec<requestedImages>> = Mutex::new(Vec::new());
}

impl Client {
    fn new(name: String, ip_address: String) -> Client {
        Client {
            name,
            ip_address
        }
    }
}
struct ClientCache {
    clients: Vec<Client>,
}

impl ClientCache {
    fn new() -> ClientCache {
        ClientCache {
            clients: Vec::new(),
        }
    }

    fn add_client(&mut self, name: String, ip_address: String) {
       
            // Client with the same name and IP address does not exist, add a new client
            self.clients.push(Client::new(name, ip_address));
        
    }

    fn find_client_by_name(&mut self, name: &str) -> Option<&mut Client> {
        self.clients.iter_mut().find(|client| client.name == name)
    }
    
}
lazy_static! {
    static ref CACHE: Mutex<ClientCache> = Mutex::new(ClientCache::new());
}

lazy_static! {
    static ref SERVER_ADDRESS: Arc<Mutex<String>> = Arc::new(Mutex::new("".to_string()));
}
lazy_static! {
    static ref EXITED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}
lazy_static! {
    static ref EXIT_FLAG: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}
fn steg_decode(path: String,i:u32) -> () {
    let encoded_image = file_as_image_buffer(path);
    let dec = Decoder::new(encoded_image);
    let out_buffer = dec.decode_alpha();
    let clean_buffer: Vec<u8> = out_buffer.into_iter()
                                  .filter(|b| {
                                      *b != 0xff_u8
                                  })
                                  .collect();
    let message = bytes_to_str(clean_buffer.as_slice());
    let path = format!("decrypted{}.png", i);
    base64_to_image(message.to_string(), path.as_str());
   
}
fn image_to_base64(path: String) -> String {
    let mut payload = File::open(path).unwrap();
    let mut payload_bytes = Vec::new();
    payload.read_to_end(&mut payload_bytes).unwrap();
    let res_base64 = base64::encode(payload_bytes);
    res_base64
 }

fn base64_to_image(base64_string: String, path: &str) {
    let decoded = base64::decode(base64_string).unwrap();
    let mut file = File::create(path).unwrap();
    file.write_all(&decoded);
}


fn read_ipaddresses() -> Vec<Ipv4Addr> {
    let path = std::path::Path::new("servers.txt");
    let mut servers = Vec::new();

    let contents = std::fs::read_to_string(path).expect("Something went wrong reading the file");
    for line in contents.lines() {
        let ipaddress: Ipv4Addr = line.parse().unwrap();
        servers.push(ipaddress);
    }
    for server in &servers {
        println!("Server: {}", server);
    }
    return servers;
}
async fn send_message_to_server(i: u32) -> Result<(), Box<dyn Error>> {
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
    let send_port = 8080;
    let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, send_port)).await?;
    let receive_port = 8082;
    let receive_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, receive_port)).await?;
    let mut buffer = [0; 1024];
    let servers = read_ipaddresses();
    let message = "2 encrypt 2";
    for server in &servers {
        println!("Sending message to server {}:{}", server, send_port);
        send_socket.send_to(message.as_bytes(), SocketAddrV4::new(*server, send_port)).await?;
    }
    println!("Listening for messages on {}:{}", local_client_address, receive_port);
    loop {
        println!("Waiting for a message...");
        let (size, addr) = receive_socket.recv_from(&mut buffer).await?;
        let received_message = String::from_utf8_lossy(&buffer[0..size]);
                let parts: Vec<&str> = received_message.trim().split(' ').collect();
                let server_address = addr.ip().to_string();
                let server_port = parts[0];
                let server_address = server_address + ":" + server_port;
                println!("server_address: {}", received_message);

                // save to global variable
                let mut send_address = SERVER_ADDRESS.lock().unwrap();
                *send_address = server_address;
                println!("Received message from {}: {}", addr, received_message);

        break;
    }
    let server_add = SERVER_ADDRESS.lock().unwrap();
    println!("server_add: {}", server_add);
    let port = server_add.split(":").collect::<Vec<&str>>();
    let mut send_image_port= 0;
    let mut send_image_address = "";
    send_image_address = port[0];
    send_image_port = port[1].parse::<u16>().unwrap();

    let mut buffer = [0; 1024];
    const MAX_CHUNK_SIZE: usize = 1024; 
    let mut image_data = Vec::new();
    // let mut file = std::fs::File::open("shanabola.jpg")?;
    let mut file = std::fs::File::open("image1.jpg")?;
    if i==0{
        file = std::fs::File::open("image1.jpg")?;
    }   
    if i==1{
        file = std::fs::File::open("image2.jpg")?;
    }
    if i==2{
        file = std::fs::File::open("image3.jpg")?;
    }
 

    file.read_to_end(&mut image_data)?;

    // Calculate the total number of chunks
    let total_chunks = (image_data.len() as f64 / MAX_CHUNK_SIZE as f64).ceil() as usize;
    println!("total_chunks: {}", total_chunks);
    println!("image_data.len(): {}", image_data.len());
    // Send image chunks over UDP
    for chunk_id in 0..total_chunks {
        let start = chunk_id * MAX_CHUNK_SIZE;
        let end = std::cmp::min((chunk_id + 1) * MAX_CHUNK_SIZE, image_data.len());
        let chunk = &image_data[start..end];
        let base64_chunk = base64::encode(chunk);

        
        let message = format!("{}:{}:{}",chunk_id, total_chunks, base64_chunk);
        for server in &servers {
            // println!("Sending message to server {}:{}", send_image_address, send_image_port);
            
            send_socket.send_to(message.as_bytes(), SocketAddrV4::new(send_image_address.parse()?, send_image_port)).await?;
        }
        // println!("Sent chunk {}/{}", chunk_id + 1, total_chunks);
    

        // Every 20 chunks, sleep for 1 second
        if chunk_id % 20 == 0 {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    // Simulate some delay between message cycles (adjust as needed)
    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("Image sent successfully!");
    let mut received_chunks: Vec<Option<Vec<u8>>> = Vec::new();
    let mut total_chunks = 0;

    loop {
        let mut buf = [0; 2048]; // Adjust the buffer size as needed
        let (num_bytes, src_addr) = receive_socket.recv_from(&mut buf).await?;

        let message = String::from_utf8_lossy(&buf[..num_bytes]);

        // Parse the message into total_chunks, chunk_id, and chunk_data
        let parts: Vec<&str> = message.trim().split(':').collect();
        if parts.len() != 3 {
            println!("Invalid message format inside rec loop: {}", message);
            eprintln!("Invalid message format: {}", message);
            continue;
        }

        let total_chunks_part: Result<usize, std::num::ParseIntError> = parts[0].parse::<usize>();
        let chunk_id_part = parts[1].parse::<usize>();

        if let (Ok(total), Ok(chunk_id)) = (total_chunks_part, chunk_id_part) {
            if total_chunks == 0 {
                total_chunks = total;
                received_chunks.resize_with(total_chunks, || None);
            }

            let chunk_data = base64::decode(parts[2]).unwrap_or_default();

            if chunk_id < total_chunks {
                received_chunks[chunk_id] = Some(chunk_data);
                //println!("Received chunk {}/{}", chunk_id + 1, total_chunks);
            }

            // Check if all chunks are received
            if received_chunks.iter().all(|x| x.is_some()) {


                let mut image_data: Vec<u8> = Vec::new();
                for chunk in received_chunks {
                    if let Some(chunk_data) = chunk {
                        image_data.extend_from_slice(&chunk_data);
                    } else {
                        eprintln!("Missing chunk");
                    }
                }
            
                // Save the reconstructed image
                // write the image buffer to a file using name encrypted{i}.png
                let path = format!("encrypted{}.png", i);
                let path2 = format!("encrypted{}.png", i);
                
                std::fs::write(path, &image_data)?;
            
                println!("Image received successfully!");
                steg_decode(path2,i); 
                break;
            }

        } else 
        {
            println!("Invalid message format inside rec else: {}", message);
            eprintln!("Invalid message format: {}", message);
        }
    }


    Ok(())
}
//ask for samples that send a message to the client asking for samples and listens on port 9098 to wait for the samples 
async fn ask_for_samples(name: String) -> Result<(), Box<dyn Error>> {
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
    let send_port=0;
    let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, send_port)).await?;
    let receive_port = 0;
    let receive_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, receive_port)).await?;
    let mut buffer = [0; 1024];
    let receive_port = receive_socket.local_addr().unwrap().port();
    // Send a message to the client
    let mut cache = CACHE.lock().unwrap();
    let client = cache.find_client_by_name(&name);
    println!("name: {}", name);

    if let Some(client) = client {
        // Client exists, send a message
        println!("Client {} found, sending a message...", name);
        let send_to_client_port = 9098;
        let message = "2";
        let message = format!("{} {} {} {}",message, "samples", "2",receive_port.to_string());
        println!("message: {}", message);
        send_socket.send_to(message.as_bytes(), SocketAddrV4::new(client.ip_address.parse()?, send_to_client_port)).await?;
        println!("message sent to {} on {}", name, client.ip_address);
        println!("waiting for client to send samples");
        drop(cache);
       
        // recieve the samples from the client and save to files as "samples1, samples2, samples2"
        let mut i=0;
        println!("Listening for messages on 9099");
        loop{
         let mut received_chunks: Vec<Option<Vec<u8>>> = Vec::new();
         let mut total_chunks = 0;
         loop{
            let mut buf = [0; 2048]; // Adjust the buffer size as needed
            let (num_bytes, src_addr) = receive_socket.recv_from(&mut buf).await?;
      
            let message = String::from_utf8_lossy(&buf[..num_bytes]);
      
            // Parse the message into total_chunks, chunk_id, and chunk_data
            let parts: Vec<&str> = message.trim().split(':').collect();
            if parts.len() != 3 {
                println!("Invalid message format inside rec loop: {}", message);
                eprintln!("Invalid message format: {}", message);
                continue;
            }
      
            let total_chunks_part: Result<usize, std::num::ParseIntError> = parts[1].parse::<usize>();
            let chunk_id_part = parts[0].parse::<usize>();
            if let (Ok(total), Ok(chunk_id)) = (total_chunks_part, chunk_id_part) {
               if total_chunks == 0 {
                   total_chunks = total;
                   received_chunks.resize_with(total_chunks, || None);
               }
      
               let chunk_data = base64::decode(parts[2]).unwrap_or_default();
      
               if chunk_id < total_chunks {
                   received_chunks[chunk_id] = Some(chunk_data);
                   println!("Received chunk {}/{}", chunk_id + 1, total_chunks);
               }
               // check u
      
               // Check if all chunks are received
               if received_chunks.iter().all(|x| x.is_some()) {
                  println!("all chunks received");
      
      
                   let mut image_data: Vec<u8> = Vec::new();
                   for chunk in received_chunks {
                       if let Some(chunk_data) = chunk {
                           image_data.extend_from_slice(&chunk_data);
                       } else {
                           eprintln!("Missing chunk");
                       }
                   }
               
                   // Save the reconstructed image
                   // write the image buffer to a file using name encrypted{i}.png
                   
                   println!("i: {}", i );
                   i=i+1;
                   let path = format!("samples{}.png", i);
                   let path2 = format!("samples{}.png", i);
                   
                   std::fs::write(path, &image_data)?;
               
                   println!("Image received successfully!");
                   break;
                  }
               }
               else {
                  println!("Invalid message format inside rec else: {}", message);
                  eprintln!("Invalid message format: {}", message);
               }

                
         }
            if i==3{
                
                break;
            }
            
      }  
    }
    else{
        println!("Client {} still not found", name);
    }
     
      

    Ok(())
}
//send samples to client 
async fn send_samples_to_client() -> Result<(), Box<dyn Error>> {
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
    let send_port=0;
    let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, send_port)).await?;
    let receive_port = 9098;
    let receive_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, receive_port)).await?;
    let mut send_to_client_port = 0;
    let mut buffer = [0; 1024];
    // let receive_port = receive_socket.local_addr().unwrap().port();
    // Send a message to the client
    let MAX_CHUNK_SIZE: usize = 1024;
    
        loop {
            let exit_flag = EXIT_FLAG.lock().unwrap();
            let exit_flag2 = exit_flag.clone();
            drop(exit_flag);
            if !exit_flag2 {
               
            
            println!("Waiting for a message...");
            let (size, addr) = receive_socket.recv_from(&mut buffer).await?;
            let received_message = String::from_utf8_lossy(&buffer[0..size]);
            println!("Received message from {}: {}", addr, received_message);
            let parts: Vec<&str> = received_message.trim().split(' ').collect();
            let client_name = parts[0];
            send_to_client_port = parts[3].parse::<u16>().unwrap();
            let mut client_name2= "Client".to_string()+client_name;

            let mut cache = CACHE.lock().unwrap();
            let mut client = cache.find_client_by_name(&client_name2);
            //clone the client name to send to the client
            
            //check if client exists in cache
            if let Some(client) = client {
                // Client exists, send a message
                println!("Client {} found, sending a message...", client_name2);
            }
            else{
                    // client does not exist, add client to cache
                    println!("Client {} not found, adding client to cache...", client_name2);
                    cache.add_client(client_name2.to_string(), addr.ip().to_string());
                    println!("Client added to cache successfully!");
                    client = cache.find_client_by_name(&client_name2);
                   
                }
            drop(cache);
            let mut file = std::fs::File::open("sample1.jpg")?;
            for mut i in 0..3{
                if i==0{
                    file = std::fs::File::open("sample1.jpg")?;
                }   
                if i==1{
                    file = std::fs::File::open("sample2.jpg")?;
                }
                if i==2{
                    file = std::fs::File::open("sample3.jpg")?;
                    // i=0;
                }
                let mut image_data = Vec::new();
                file.read_to_end(&mut image_data)?;
                // Calculate the total number of chunks
                let total_chunks = (image_data.len() as f64 / MAX_CHUNK_SIZE as f64).ceil() as usize;
                println!("total_chunks: {}", total_chunks);
                println!("image_data.len(): {}", image_data.len());
                // Send image chunks over UDP
                for chunk_id in 0..total_chunks {
                    let start = chunk_id * MAX_CHUNK_SIZE;
                    let end = std::cmp::min((chunk_id + 1) * MAX_CHUNK_SIZE, image_data.len());
                    let chunk = &image_data[start..end];
                    let base64_chunk = base64::encode(chunk);
                    let message = format!("{}:{}:{}",chunk_id, total_chunks, base64_chunk);
                    println!("Sending chunk {}/{}", chunk_id + 1, total_chunks);
                    //get cache
                    let mut cache = CACHE.lock().unwrap();
                    let client = cache.find_client_by_name(&client_name2);
                    send_socket.send_to(message.as_bytes(), SocketAddrV4::new(client.unwrap().ip_address.parse()?, send_to_client_port)).await?;
                    drop(cache);
                    // Every 20 chunks, sleep for 1 second
                    if chunk_id % 20 == 0 {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }
                println!("sample {} sent", i + 1);
                println!("Image sent successfully!");
                if i==2{
                    i=0;
                    break;
                    
                }
            }
            
        }
    }
}
//function takes in from the terminal "samples , annd the client name I am requesting the samples from", 
async fn check_client_ip(name:String, command:String, image_id:String) -> Result<(), Box<dyn Error>> {
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
    let send_port=0;
    let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, send_port)).await?;
    let mut request_name = name.clone();
    
    // check if client exists in cache
    println!("Checking if client exists in cache...");
    println!("name: {}", name);
    let mut cache = CACHE.lock().unwrap();
    let client = cache.find_client_by_name(&name);
    println!("bye");
    if let Some(client) = client {
     
        // Client exists, send a message
        println!("Client {} found, sending a message...", name);
        // let send_to_client_port = 9090;
        // let message = format!("{} {}", "1 request", name);
        // let message = format!("{}:{}", name, message);
        // send_socket.send_to(message.as_bytes(), SocketAddrV4::new(client.ip_address.parse()?, send_to_client_port)).await?;
        // println!("Message sent successfully!");
    } else {
        // Client does not exist, send to server to get the client's IP address on port 9091
        println!("Client {} not found, sending a request to the server...", name);
        let servers = read_ipaddresses();
        let server_port = 8080;
        let message = format!("{} {}", "2 request", name);

        for server in &servers {
            println!("Sending message to server {}:{}", server, server_port);
            println!("message: {}", message);
            send_socket.send_to(message.as_bytes(), SocketAddrV4::new(*server, server_port)).await?;
        }
        println!("Request sent to servers successfully!");
        //wait for response on port 8082
        let receive_port = 8082;
        let receive_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, receive_port)).await?;
        let mut buffer = [0; 1024];
        println!("Listening for messages on {}:{}", local_client_address, receive_port);
        loop {
            println!("Waiting Server to send requested ip...");
            let (size, addr) = receive_socket.recv_from(&mut buffer).await?;
            let received_message = String::from_utf8_lossy(&buffer[0..size]);
            println!("received_message: {}", received_message);
            // create client and add to cache with name and ip address from message
            if received_message=="none"{
                println!("Client not found");
                break;
            }
            cache.add_client(request_name.clone(), received_message.to_string());
            println!("new client added to cache");
            println!("Received message from {}: {}", addr, received_message);
            break;
        }
    }
        // Send a message to the client without blocking the main thread
        if command=="request"{
            let send_message_handle = thread::spawn(move || {
                if let Err(err) = async_std::task::block_on(send_message_to_client(request_name, image_id)) {
                    eprintln!("Error in sending messages to client (request): {}", err);
                }
            });
        }
        else if command=="samples"{
         println!("sending now");
           
                let send_message_handle = thread::spawn(move || {

                    if let Err(err) = async_std::task::block_on(ask_for_samples(request_name.clone())) {
                        eprintln!("Error in sending messages to client (samples): {}", err);
                    }
                });
            
        }
        // else if command=="request_more_access_rights"{
        //     let send_message_handle = thread::spawn(move || {
        //         if let Err(err) = async_std::task::block_on(request_more_access_rights(request_name, image_id, "5".to_string())) {
        //             eprintln!("Error in sending messages to client request_more_access_rights: {}", err);
        //         }
        //     });
        // }
        // else if command=="access"{
        //     let send_message_handle = thread::spawn(move || {
        //         if let Err(err) = async_std::task::block_on(request_more_access_rights(request_name, image_id, "5".to_string())) {
        //             eprintln!("Error in sending messages to client access: {}", err);
        //         }
        //     });
        // }
        else{
            println!("invalid command");
        }
        // let send_message_handle = thread::spawn(move || {
        //     if let Err(err) = async_std::task::block_on(send_message_to_client(request_name, image_id)) {
        //         eprintln!("Error in sending messages to client: {}", err);
        //     }
        // });
        println!("hola");
     
       
    
      drop(cache);

    Ok(())
}
async fn send_message_to_client(name: String, image_id:String) -> Result<(), Box<dyn Error>> {
    //put the name and image id in requested images
    let mut requested_images = REQUESTED_IMAGES.lock().unwrap();
    requested_images.push(requestedImages{image_id: image_id.clone(), sender_client_name: name.clone(), access_rights: "0".to_string(), image_data: Vec::new()});
    drop(requested_images);
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
    let send_port=0;
    let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, send_port)).await?;
    let receive_port = 0;
    let receive_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, receive_port)).await?;
    let mut buffer = [0; 1024];
    let receive_port = receive_socket.local_addr().unwrap().port();
    // Send a message to the client
    let mut cache = CACHE.lock().unwrap();
    let client = cache.find_client_by_name(&name);
    if let Some(client) = client {
        // Client exists, send a message
        println!("Client {} found, sending a message...", name);
        let send_to_client_port = 9090;
        let message = "2";
        let message = format!("{} {} {}", receive_port,message, image_id.clone());
        println!("message: {}", message);
        send_socket.send_to(message.as_bytes(), SocketAddrV4::new(client.ip_address.parse()?, send_to_client_port)).await?;
        println!("waiting for client to send image");
        let mut received_chunks: Vec<Option<Vec<u8>>> = Vec::new();
        let mut total_chunks = 0;
    
        loop {

            let mut buf = [0; 2048]; // Adjust the buffer size as needed
            let (num_bytes, src_addr) = receive_socket.recv_from(&mut buf).await?;
    
            let message = String::from_utf8_lossy(&buf[..num_bytes]);
    
            // Parse the message into total_chunks, chunk_id, and chunk_data
            let parts: Vec<&str> = message.trim().split(':').collect();
            if parts.len() != 3 {
                println!("first msg !=3");
                if parts.len()== 4{
                    println!("first msg ==4");
                   //push in requested images
                     println!("after lock");
                     let mut requested_images = REQUESTED_IMAGES.lock().unwrap();
                     //find the image id (parts 1) and client name (parts 2) in requested images, then change access rights to parts 3
                        for i in 0..requested_images.len(){
                            //print the image id and client name\
                            println!("image id: {}", requested_images[i].image_id);
                            println!("image id: {}", parts[2]);
                            println!("client name: {}", requested_images[i].sender_client_name);
                            println!("client name: {}", parts[1]);
                            if requested_images[i].image_id==parts[2] && requested_images[i].sender_client_name==parts[1]{
                                requested_images[i].access_rights=parts[3].to_string();
                                println!("access rights changed to {}", parts[3]);
                            }
                        }
                        drop(requested_images);
                }
                else{
                    println!("Invalid message format inside rec loop: {}", message);
                    eprintln!("Invalid message format: {}", message);
                    continue;
                }
            }else{
                let total_chunks_part: Result<usize, std::num::ParseIntError> = parts[1].parse::<usize>();
                let chunk_id_part = parts[0].parse::<usize>();

           
    
                if let (Ok(total), Ok(chunk_id)) = (total_chunks_part, chunk_id_part) {
                    if total_chunks == 0 {
                        total_chunks = total;
                        received_chunks.resize_with(total_chunks, || None);
                    }
        
                    let chunk_data = base64::decode(parts[2]).unwrap_or_default();
        
                    if chunk_id < total_chunks {
                        received_chunks[chunk_id] = Some(chunk_data);
                        // println!("Received chunk {}/{}", chunk_id + 1, total_chunks);
                    }
        
                    // Check if all chunks are received
                    if received_chunks.iter().all(|x| x.is_some()) {
                        println!("all chunks received");
        
                        let mut image_data: Vec<u8> = Vec::new();
                        for chunk in received_chunks {
                            if let Some(chunk_data) = chunk {
                                image_data.extend_from_slice(&chunk_data);
                            } else {
                                eprintln!("Missing chunk");
                            }
                        }
                    
                        // Save the reconstructed image
                        // write the image buffer to a file using name encrypted{i}.png
                        // let path = format!("Recieved_from_client.png");
                        // push image data to requested images
                        let mut requested_images = REQUESTED_IMAGES.lock().unwrap();
                        for i in 0..requested_images.len(){
                            if requested_images[i].image_id==image_id && requested_images[i].sender_client_name==name{
                                requested_images[i].image_data=image_data.clone();
                                println!("image data pushed to requested images");
                                println!("image data: {}", requested_images[i].image_data.len());
                            }
                        }
                        drop(requested_images);
                        // std::fs::write(path, &image_data)?;
                    
                        println!("Image received successfully!");
                        break;
                    }
        
                } else 
                {
                    println!("Invalid message format inside rec else: {}", message);
                    eprintln!("Invalid message format: {}", message);
                }

            }   
        }
    } else {
        println!("Client {} still not found", name);
    }
    drop(cache);

    Ok(())

    
    
}

async fn receive_message_from_client() -> Result<(), Box<dyn Error>> {
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
    let receive_port = 9090;
    let receive_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, receive_port)).await?;
    let mut buffer = [0; 1024];
    println!("Listening for messages on {}:{}", local_client_address, receive_port);
    loop {
        println!("Waiting for a message...");
        let exit_flag = EXIT_FLAG.lock().unwrap();
        let exit_flag2 = exit_flag.clone();
        drop(exit_flag);
        if !exit_flag2 {
        let (size, addr) = receive_socket.recv_from(&mut buffer).await?;
        let received_message = String::from_utf8_lossy(&buffer[0..size]);
        
        println!("Received message from {}: {}", addr, received_message);
        let parts: Vec<&str> = received_message.trim().split(' ').collect();
        let send_port = parts[0];
        let client_name = parts[1];
        let mut client_name2= "Client".to_string()+client_name;
        let image_id = parts[2];
        let access_rights = "5";
        let mut requesting_clients = REQUESTING_CLIENTS.lock().unwrap();
        requesting_clients.push(requestingClient{image_id: image_id.to_string(), requesting_client_name: client_name2.to_string(), access_rights: access_rights.to_string()});
        drop(requesting_clients);
        let mut cache = CACHE.lock().unwrap();
        let client = cache.find_client_by_name(&client_name2);
        // check if exit flag is true
        
           
            
        
        
        if let Some(client) = client {
            // client already exists, send image to client
            println!("Client {} found, sending image...", client_name2);
         }
        else{
                // client does not exist, add client to cache
                println!("Client {} not found, adding client to cache...", client_name2);
                cache.add_client(client_name2.to_string(), addr.ip().to_string());
                println!("Client added to cache successfully!");
               
            }
            drop(cache);
        let send_to_client_port = send_port.parse::<u16>().unwrap();
        let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, 0)).await?;
        let mut buffer = [0; 1024];
        const MAX_CHUNK_SIZE: usize = 1024; 
        let mut image_data = Vec::new();
        let mut file = std::fs::File::open("encrypted0.png")?;
        if image_id=="1"{
            file = std::fs::File::open("encrypted0.png")?;
        }
        if image_id=="2"{
            file = std::fs::File::open("encrypted1.png")?;
        }
        if image_id=="3"{
            file = std::fs::File::open("encrypted2.png")?;
        }

        file.read_to_end(&mut image_data)?;

        // Calculate the total number of chunks
        let total_chunks = (image_data.len() as f64 / MAX_CHUNK_SIZE as f64).ceil() as usize;
        println!("total_chunks: {}", total_chunks);
        println!("image_data.len(): {}", image_data.len());
        // Send image chunks over UDP
        let message2= format!("{}:{}:{}:{}", "first_message", "Client2", image_id, access_rights);
        send_socket.send_to(message2.as_bytes(), SocketAddrV4::new(addr.ip().to_string().parse()?, send_to_client_port)).await?;

        for chunk_id in 0..total_chunks {
            let start = chunk_id * MAX_CHUNK_SIZE;
            let end = std::cmp::min((chunk_id + 1) * MAX_CHUNK_SIZE, image_data.len());
            let chunk = &image_data[start..end];
            let base64_chunk = base64::encode(chunk);
            let message = format!("{}:{}:{}",chunk_id, total_chunks, base64_chunk);
            //println!("Sending chunk {}/{}", chunk_id + 1, total_chunks);
            send_socket.send_to(message.as_bytes(), SocketAddrV4::new(addr.ip().to_string().parse()?, send_to_client_port)).await?;
        

            // Every 20 chunks, sleep for 1 second
            if chunk_id % 20 == 0 {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        let client_name = parts[1];
        println !("Image sent successfully to {}", client_name);
    }
}
}

async fn send_exit_message_to_server() -> Result<(), Box<dyn Error>> {
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
    let send_port = 8080;
    let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, send_port)).await?;

    let servers = read_ipaddresses();
    let message = "1 exit 1";
    for server in &servers {
        println!("Sending message to server {}:{}", server, send_port);
        send_socket.send_to(message.as_bytes(), SocketAddrV4::new(*server, send_port)).await?;
    }

    Ok(())
}
async fn listener_to_client_ack() -> Result<String, Box<dyn Error>> {
    println!("listening to client ack");
    // receive acknoledgement from client
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
    let receive_port = 7070;
    let receive_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, receive_port)).await?;
    let mut buffer = [0; 1024];
    println!("Listening for messages on {}:{}", local_client_address, receive_port);

    let mut buffer = vec![0u8; 1024];
    if let Ok((len, remote_addr)) = receive_socket.recv_from(&mut buffer).await {
        let remote_ip = remote_addr.ip().to_string();
        println!("Received msg from CLIENT; IP = {}", remote_ip);
        
        let received_message = String::from_utf8_lossy(&buffer[..len]).to_string();
        println!("received_message is {}", received_message);
        // Clear the buffer for the next message.
        buffer.clear();
        return Ok(received_message);

    }


        Err("No message received".into())

 }


 #[tokio::main]
async fn change_access_rights(client_name: String, image_id: String, access_rights: String) -> Result<(), Box<dyn Error>> {
    let mut requesting_clients = REQUESTING_CLIENTS.lock().unwrap();
    //change access rights in requesting client wuth client name and image id
    for i in 0..requesting_clients.len(){
        if requesting_clients[i].image_id==image_id && requesting_clients[i].requesting_client_name==client_name{
            requesting_clients[i].access_rights=access_rights.to_string();
            println!("access rights changed to {}", access_rights);
        }
    }
    drop(requesting_clients);
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
    let send_port = 0;
    let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, send_port)).await?;

    //send message to client_name with access_rights
    let mut cache = CACHE.lock().unwrap();
    let client = cache.find_client_by_name(&client_name);
   
    if let Some(client) = client {

        // Client exists, send a message
        println!("Client {} found, sending a message...", client_name);
        let send_to_client_port = 9094;
        let message = format!("{}:{}:{}", "2", image_id, access_rights);
        send_socket.send_to(message.as_bytes(), SocketAddrV4::new(client.ip_address.parse()?, send_to_client_port)).await?;
        println!("Message sent successfully!");
    } 
    drop(cache);
    let result = tokio::time::timeout(Duration::from_secs(10), listener_to_client_ack()).await;
    match result {
        Ok(Ok(received_message)) => {
            println!("Received message from client: {}", received_message);
            if received_message=="1"{
                println!("access rights changed successfully");
            }
            else{
                println!("access rights not changed");
            }
        }
        Ok(Err(err)) => {
            println!("Error in receiving message from client: {}", err);
        }
        Err(err) => {
            println!("Timeout in receiving message from client: {}", err);
            // send  to server to change access rights
            let servers = read_ipaddresses();
            let server_port = 8080;
            let message = format!("{} {} {} {} {}", "2", "unhandled", client_name, image_id, access_rights);
            println!("message: {}", message);
            for server in &servers {
                println!("Sending message to server {}:{}", server, server_port);
                send_socket.send_to(message.as_bytes(), SocketAddrV4::new(*server, server_port)).await?;
            }

        }
    }
    
        

    Ok(())
}


async fn receive_access_rights_from_client1() -> Result<(), Box<dyn Error>> {
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
    let receive_port = 9095;
    let receive_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, receive_port)).await?;
    let mut buffer: [u8; 1024] = [0; 1024];
    println!("Listening for messages on {}:{}", local_client_address, receive_port);
    loop {
        println!("Waiting for a message...");
        println!("check 1");
        
        let (size, addr) = receive_socket.recv_from(&mut buffer).await?;
        let received_message = String::from_utf8_lossy(&buffer[0..size]);
        let exit_flag = EXIT_FLAG.lock().unwrap();
        let exit_flag2 = exit_flag.clone();
        drop(exit_flag);
        println!("exit flag is {}", exit_flag2);
        if !exit_flag2 {
            println!("exit flag is false");
        
        println!("Received message from {}: {}", addr, received_message);
        let parts: Vec<&str> = received_message.trim().split(':').collect();
        let client_id: &str = parts[0];
        let mut client_id2= "Client".to_string()+client_id;
        let image_id = parts[1];
        let access_rights = parts[2];
        let mut requested_images = REQUESTED_IMAGES.lock().unwrap();
        for i in 0..requested_images.len(){
            if requested_images[i].image_id==image_id && requested_images[i].sender_client_name==client_id2{
                requested_images[i].access_rights=access_rights.to_string();
                println!("access rights changed to {}", access_rights);
            }
        }
        drop(requested_images);
        // send acknowledgment to client on port 7070
        let send_port = 7070;
        let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, send_port)).await?;
        let message = "1";
        //get address without the port number
        let mut addr2 = addr.to_string();
        let mut addr3 = addr2.split(':').collect::<Vec<&str>>();
        let addr4 = addr3[0];

        send_socket.send_to(message.as_bytes(), SocketAddrV4::new(addr4.parse()?, 7070)).await?;
        }
    }

}
async fn receive_access_rights_from_client3() -> Result<(), Box<dyn Error>> {
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
    let receive_port = 9096;
    let receive_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, receive_port)).await?;
    let mut buffer: [u8; 1024] = [0; 1024];
    println!("Listening for messages on {}:{}", local_client_address, receive_port);
    loop {
        println!("Waiting for a message...");
        println!("check 1");
        
        let (size, addr) = receive_socket.recv_from(&mut buffer).await?;
        let received_message = String::from_utf8_lossy(&buffer[0..size]);
        let exit_flag = EXIT_FLAG.lock().unwrap();
        let exit_flag2 = exit_flag.clone();
        drop(exit_flag);
        println!("exit flag is {}", exit_flag2);
        if !exit_flag2 {
            println!("exit flag is false");
        
        println!("Received message from {}: {}", addr, received_message);
        let parts: Vec<&str> = received_message.trim().split(':').collect();
        let client_id: &str = parts[0];
        let mut client_id2= "Client".to_string()+client_id;
        let image_id = parts[1];
        let access_rights = parts[2];
        let mut requested_images = REQUESTED_IMAGES.lock().unwrap();
        for i in 0..requested_images.len(){
            if requested_images[i].image_id==image_id && requested_images[i].sender_client_name==client_id2{
                requested_images[i].access_rights=access_rights.to_string();
                println!("access rights changed to {}", access_rights);
            }
        }
        drop(requested_images);
        // send acknowledgment to client on port 7070
        let send_port = 7070;
        let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, send_port)).await?;
        let message = "1";
        //get address without the port number
        let mut addr2 = addr.to_string();
        let mut addr3 = addr2.split(':').collect::<Vec<&str>>();
        let addr4 = addr3[0];

        send_socket.send_to(message.as_bytes(), SocketAddrV4::new(addr4.parse()?, 7070)).await?;
        }
    }

}

async fn request_more_access_rights(client_name: String, image_id: String, access_rights: String) -> Result<(), Box<dyn Error>> {
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
      let send_port=0;
      let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, send_port)).await?;
      let mut receive_port = 0;
      let receive_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, receive_port)).await?;
        receive_port= receive_socket.local_addr().unwrap().port();
        let send_to_client_port = 9097;
      let mut buffer = [0; 1024];
      // Send a message to the client
      let mut cache = CACHE.lock().unwrap();
      let client = cache.find_client_by_name(&client_name);
      if let Some(client) = client {
          // Client exists, send a message
          println!("Client {} found, sending a message...", client_name);
          let message = "2";
          let message = format!("{} {} {} {}", message,image_id.clone(),access_rights.clone(), receive_port);
          println!("message: {}", message);
          send_socket.send_to(message.as_bytes(), SocketAddrV4::new(client.ip_address.parse()?, send_to_client_port)).await?;
          println!("waiting for client to reply to request");
      
      
          loop {
  
              let mut buf = [0; 2048]; // Adjust the buffer size as needed
              let (num_bytes, src_addr) = receive_socket.recv_from(&mut buf).await?;
      
              let message = String::from_utf8_lossy(&buf[..num_bytes]);
              // check if message is negative or positive message is only a number "1"
 
             let mut received_number = message.clone();
             let mut received_number2 = received_number.trim().parse::<i32>().unwrap();
             if received_number2<0{
                 println!("access rights not granted");
                 break;
             }
             else{
                   println!("access rights granted");
                   //change access rights in requested images
                   let mut requested_images = REQUESTED_IMAGES.lock().unwrap();
                   for i in 0..requested_images.len(){
                      if requested_images[i].image_id==image_id && requested_images[i].sender_client_name==client_name{
                            requested_images[i].access_rights=access_rights.to_string();
                            println!("access rights changed to {}", access_rights);
                      }
                   }
                   drop(requested_images);
                   break;
             }
            }
         }
         Ok(())
 } 
 async fn receive_request_for_more_access_rights() -> Result<(), Box<dyn Error>> {
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
      let receive_port = 9097;
      let receive_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, receive_port)).await?;
      let send_port=0;
      let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, send_port)).await?;
      let mut buffer = [0; 1024];
      println!("Listening for messages on {}:{}", local_client_address, receive_port);
      loop{
         let mut buf = [0; 2048]; // Adjust the buffer size as needed
         let exit_flag = EXIT_FLAG.lock().unwrap();
         let exit_flag2 = exit_flag.clone();
         drop(exit_flag);
         if !exit_flag2 {
         let (num_bytes, src_addr) = receive_socket.recv_from(&mut buf).await?;
 
         let message = String::from_utf8_lossy(&buf[..num_bytes]);
        
         println!("Received message from {}: {}", src_addr, message);
         
         // parse message into client id, image id, and access rights
         let parts: Vec<&str> = message.trim().split(' ').collect();
         let client_id: &str = parts[0];
         let mut client_id2 = "Client".to_string()+ client_id;
         let image_id = parts[1];
         let access_rights = parts[2];
         let send_to_client_port = parts[3].parse::<u16>().unwrap();
         //check if client exists in cache
         println!("client id: {}", client_id2);
         println!("image id: {}", image_id);
        println!("access rights: {}", access_rights);

         let mut cache = CACHE.lock().unwrap();
         let client = cache.find_client_by_name(&client_id2);
         if let Some(client) = client {
            // promt user to accept or reject request
            println!("Client {} found, sending a message...", client_id2);
            println!("Client {} requested more access rights to image {}", client_id2, image_id);
            println!("Do you want to grant access rights of value {} to {} to image {}? (y/n)", access_rights, client_id2, image_id);
            let mut user_input = String::new();
            io::stdin().read_line(&mut user_input)?;
            let user_input = user_input.trim();
            if user_input=="y"{
                //send positive message to client
                let message = "1";
                let message = format!("{}",access_rights.clone());
                println!("message: {}", message);
                
                send_socket.send_to(message.as_bytes(), SocketAddrV4::new(client.ip_address.parse()?, send_to_client_port)).await?;
                println!("access rights granted");
                println!("access_rights that the client agreed to: {}", access_rights);
                //change access rights in requested images
                let mut requestingClients = REQUESTING_CLIENTS.lock().unwrap();
                  for i in 0..requestingClients.len(){
                     if requestingClients[i].image_id==image_id && requestingClients[i].requesting_client_name==client_id2{
                           requestingClients[i].access_rights=access_rights.to_string();
                           println!("access rights changed to {}", access_rights);
                     }
                  }
                drop(requestingClients);
            }
            else{
                let message = "-1";
                let message = format!("{}", message);
                println!("message: {}", message);
                send_socket.send_to(message.as_bytes(), SocketAddrV4::new(client.ip_address.parse()?, send_to_client_port)).await?;
                println!("access rights not granted");
            }


         }
         drop(cache);
      }
    }
}
async fn send_online_message_to_server() -> Result<(), Box<dyn Error>> {
    let local_client_address: Ipv4Addr = "127.0.0.5".parse()?;
    let send_port = 8080;
    let send_socket = UdpSocket::bind(SocketAddrV4::new(local_client_address, send_port)).await?;
    let message = "2 online 2";
    let servers = read_ipaddresses();
    for server in &servers {
        println!("Sending message to server {}:{}", server, send_port);
        send_socket.send_to(message.as_bytes(), SocketAddrV4::new(*server, send_port)).await?;
    }
    Ok(())

}   

fn pop_image(path: String) -> Result<(), Box<dyn Error>>
 {

    let path2 = path.clone();
    let img = image::open(path)?;
    // Convert the image to a byte slice.
    let bytes = img.to_bytes();
    
    // Convert the byte slice to an ImageView.
    let image_view = ImageView::new(ImageInfo::rgb8(img.width(), img.height()), &bytes);
    
    // Create a window with default options and display the image.
    let window = create_window("image", Default::default())?;
    window.set_image(path2, image_view)?;
    
    Ok(())
}
fn view_image(image_id: String,owner_client_name: String) -> Result<(), Box<dyn Error>> {
    println!("view image");
    let mut requested_images = REQUESTED_IMAGES.lock().unwrap();
    println!("1 lock");
    //loop on requested images, when the image id and the owner client name is found then decrement the access rights by 1 
    for i in 0..requested_images.len(){
        println!("inisdde loop");
        if requested_images[i].image_id==image_id && requested_images[i].sender_client_name==owner_client_name{
            println!("inside if");
            let mut access_rights2 = requested_images[i].access_rights.parse::<i32>().unwrap();
            access_rights2=access_rights2-1;
            requested_images[i].access_rights=access_rights2.to_string();
         //get image data from requested images
            let image_data = requested_images[i].image_data.clone();
            //write image data to a file
            let path2 = "encrypted.png";
            std::fs::write(path2, &image_data)?;
            let encoded_image = file_as_image_buffer(path2.to_string());
            let dec = Decoder::new(encoded_image);
            let out_buffer = dec.decode_alpha();
            let clean_buffer: Vec<u8> = out_buffer.into_iter()
                                          .filter(|b| {
                                              *b != 0xff_u8
                                          })
                                          .collect();
            let message = bytes_to_str(clean_buffer.as_slice());
            let image_path = "decrypted.jpg";
            println!("lol: -1");
            base64_to_image(message.to_string(),image_path);
            println!("lol: 0");
            pop_image(image_path.to_string())?;
            //delete decrypted.jpg
            std::fs::remove_file(image_path)?;
            
        }
    }

    drop(requested_images);
    Ok(())

}
#[show_image::main]  
fn main() -> Result<(), Box<dyn std::error::Error>> {
  

    let online_handler_handle = thread::spawn(move || {
        if let Err(err) = async_std::task::block_on(send_online_message_to_server()) {
            eprintln!("Error in sending online message to server: {}", err);
        }
    });
   
    let client_listener_handle = thread::spawn(move || {
       if let Err(err) = async_std::task::block_on(receive_message_from_client()) {
             eprintln!("Error in receiving messages from client: {}", err);
       }
    });
    let access_rights_listener_handle = thread::spawn(move || {
          println!("access rights listener");
          if let Err(err) = async_std::task::block_on(receive_access_rights_from_client3()) {
             eprintln!("Error in receiving messages from client: {}", err);
          }
    });
    let request_more_access_rights_listener_handle = thread::spawn(move || {
          println!("request more access rights listener");
          if let Err(err) = async_std::task::block_on(receive_request_for_more_access_rights()) {
             eprintln!("Error in receiving messages from client: {}", err);
          }
    });
    let access_rights_listener_handle2 = thread::spawn(move || {
        println!("access rights listener");
        if let Err(err) = async_std::task::block_on(receive_access_rights_from_client1()) {
           eprintln!("Error in receiving messages from client: {}", err);
        }
    });
    //call function to send samples to client
      let send_samples_handle = thread::spawn(move || {
          println!("send samples");
          if let Err(err) = async_std::task::block_on(send_samples_to_client()) {
             eprintln!("Error in sending messages to client  send_samples_handle: {}", err);
          }
      });
      let mut previous_command = String::new();
      
    loop {

      print!("Enter a command: ");
      io::stdout().flush()?; // Flush the stdout to ensure the prompt is displayed

      let mut user_input = String::new();
      io::stdin().read_line(&mut user_input)?;

      // Trim any leading or trailing whitespaces from user input
      // check if input is more than one word
      let mut args: Vec<&str> = user_input.trim().split(' ').collect();
      let command = args[0];
      let command = command.trim();
      
      



      // Perform actions based on the user command
      match command {
          "access" => {
                if previous_command=="exit"{
                    println!("Client is offline, type 'up' to turn back on");
                }
                else{
                    previous_command= command.clone().to_string();
                    if args.len() == 4 {
                        let client_name = &args[1];
                        let image_id= &args[2];
                        let access_rights= &args[3];
                        println!("Changing access rights of image: {} from client: {} to {}", image_id , client_name, access_rights);
                        change_access_rights(client_name.to_string(), image_id.to_string(), access_rights.to_string())?;
      
                    } else {
                        println!("Usage: {} access <client_name> <image_id> <access_rights>", args[0]);
                    }

                }
          }
          "request" => {
            if previous_command=="exit"{
                println!("Client is offline, type 'up' to turn back on");
            }
            else{
                previous_command= command.clone().to_string();
                
                if args.len() == 3 {
                    let client_name = &args[1];
                    let image_id= &args[2];
                    println!("Requesting image: {} from client: {}", image_id , client_name);
                    async_std::task::block_on(check_client_ip(client_name.to_string(), command.to_string(), image_id.to_string()))?;
                    
      
                } else {
                    println!("Usage: {} request <client_name>", args[0]);
                }
            }

          
          }

          "encrypt" => {
             //check if previous command was exit , if it was then print client is offline and dont encrypt
             println!("previous command: {}", previous_command);
                if previous_command=="exit"{
                    println!("Client is offline, type 'up' to turn back on");
                }
                else{
                    previous_command= command.clone().to_string();
                    let start_time = Instant::now();
                    for i in 0..3{
                        let send_multicast_handle = thread::spawn(move || {
                            if let Err(err) = async_std::task::block_on(send_message_to_server(i)) {
                                eprintln!("Error in sending multicast messages: {}", err);
                            }
                        });
                        send_multicast_handle.join().expect("Multicast thread panicked");   
                    }
                   
                    let elapsed_time = start_time.elapsed();
                    let elapsed_secs = elapsed_time.as_secs_f64();
                    
                    println!("Total time elapsed: {:.2} seconds", elapsed_secs);
                    println!("Average time per image: {:.2} seconds", elapsed_secs/3.0);
                }

          }
          "ask" => {
                //check if previous command was exit , if it was then print client is offline and dont ask
                if previous_command=="exit"{
                    println!("Client is offline, type 'up' to turn back on");
                }
                else{
                    
                    if args.len() == 4 {
                        let client_name = &args[1];
                        let image_id= &args[2];
                        let access_rights= &args[3];
                        println!("Requesting more access rights of image: {} from client: {} to {}", image_id , client_name, access_rights);
                        async_std::task::block_on(request_more_access_rights(client_name.to_string(), image_id.to_string(), access_rights.to_string()))?;
                        
      
                     } else {
                        println!("Usage: {} access <client_name> <image_id> <access_rights>", args[0]);
                     }
                     previous_command= command.clone().to_string();
                }
          }
          "samples"=>{
            //  check if previous command was exit , if it was then print client is offline and dont ask
            if previous_command=="exit"{
                println!("Client is offline, type 'up' to turn back on");
            }
            else{
                if args.len() == 2 {
                    let client_name = &args[1];
                    println!("Requesting samples from client: {}", client_name);
                    async_std::task::block_on(check_client_ip(client_name.to_string(), command.to_string(), "".to_string()))?;
                    
    
                } else {
                    println!("Usage: {} request <client_name>", args[0]);
                }
            }
          }
          "up" => {
             let send_online_message_handle = thread::spawn(move || {
                 if let Err(err) = async_std::task::block_on(send_online_message_to_server()) {
                     eprintln!("Error in sending exit messages: {}", err);
                 }
             });
             previous_command= command.clone().to_string();
             let mut exit_flag = EXIT_FLAG.lock().unwrap();
                *exit_flag = false;
                drop(exit_flag);
                println!("The Client is Now online, type 'exit' to turn back off");
              
          }
          "view" => {
            // Get args of image id and client name
            if args.len() == 3 {
                let owner_client_name = &args[1];
                let image_id = &args[2];
                println!("Viewing image: {} from client: {}", image_id, owner_client_name);
                // Check if image exists in requested images
                let mut requested_images = REQUESTED_IMAGES.lock().unwrap();
                let mut image_found = false;
        
                for i in 0..requested_images.len() {
                    if requested_images[i].image_id == image_id.to_string()
                        && requested_images[i].sender_client_name == owner_client_name.to_string()
                    {
                        image_found = true;
                        println!("Image found");
                        // Check if access rights is 0
                        if requested_images[i].access_rights == "0" {
                            println!("You do not have access rights to this image");
                            
                            
                        } else {
                            let image_id = requested_images[i].image_id.clone();
                            let owner = requested_images[i].sender_client_name.clone();
                            drop(requested_images);
                            view_image(
                                image_id,
                                owner,
                            )?;
                    
                            break;
                        }
                        
                    }
                }
        
                if !image_found {
                    println!("Image not found");
                }
        
                // Drop requested_images outside of the loop
              
            } else {
                println!("Usage: {} view <client_name> <image_id>", args[0]);
            }
        }
        
          "exit" => {
             // SET exit flag to true
                let mut exit_flag = EXIT_FLAG.lock().unwrap();
                *exit_flag = true;
                drop(exit_flag);
                println!("The Client is Now offline, type 'up'to turn back on");
                previous_command= command.clone().to_string();

         }
         
         _ => {
             println!("Unknown command: {}", command);
         }
      } 
  }


//    client_listener_handle.join().expect("Client listener thread panicked");
//    access_rights_listener_handle.join().expect("Client listener thread panicked");
//    request_more_access_rights_listener_handle.join().expect("Client listener thread panicked");
//    send_samples_handle.join().expect("Client listener thread panicked");

//   Ok(())
}