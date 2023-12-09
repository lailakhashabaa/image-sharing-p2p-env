use async_std::net::{UdpSocket, SocketAddrV4, ToSocketAddrs};
use sysinfo::DiskUsage;
use std::error::Error;
use std::net::Ipv4Addr;
use std::thread;
use sysinfo::{ProcessExt, System, SystemExt, PidExt,ProcessStatus};
use std::sync::{Arc, Mutex};
use std::net:: SocketAddr;
use std::time::Duration;
use tokio::time::sleep;
use std::time::Instant;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use steganography::encoder::*;
use steganography::util::*;
use async_std::task;
extern crate image;
use image::{DynamicImage, ImageBuffer, Rgba};
use image::GenericImageView;
use image::{ ImageOutputFormat};
use std::io::Cursor;
use base64;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use async_std::prelude::StreamExt;



#[derive(Debug)]
#[derive(Clone)]

//create a struct that saves requesting client name, owning client name, image id, and the request 

struct PendingRequest {
    id: u32,
    address: Ipv4Addr,
    request_type: String,
    message: String,
}

impl PendingRequest {
    // Method to remove a request by index
    fn remove_request_by_index(requests: &mut Vec<PendingRequest>, index: usize) {
        if let Some(removed_request) = requests.get(index).cloned() {
            requests.remove(index);
            println!("Removed request at index {}: {:?}", index, removed_request);
        } else {
            println!("Index {} out of bounds or no requests to remove.", index);
        }
    }
}
lazy_static! {
    static ref PENDING_REQUESTS: Arc<Mutex<Vec<PendingRequest>>> = Arc::new(Mutex::new(Vec::new()));
}
#[derive(Debug)]
#[derive(Clone)]
struct Request {
    requesting_client_name: String,
    owning_client_name: String,
    image_id: String,
    request: String,
}

impl Request {
    // Method to remove a request by index
    fn remove_request_by_index(requests: &mut Vec<Request>, index: usize) {
        if let Some(removed_request) = requests.get(index).cloned() {
            requests.remove(index);
            println!("Removed request at index {}: {:?}", index, removed_request);
        } else {
            println!("Index {} out of bounds or no requests to remove.", index);
        }
    }
    //add request to requests
    fn add_request(&mut self, requesting_client_name: String, owning_client_name: String, image_id: String, request: String) {
        self.requesting_client_name = requesting_client_name;
        self.owning_client_name = owning_client_name;
        self.image_id = image_id;
        self.request = request;
    }
}
lazy_static! {
    static ref REQUESTS: Arc<Mutex<Vec<Request>>> = Arc::new(Mutex::new(Vec::new()));
}
lazy_static! {
    static ref CURRENT_ONLINE_SENDER: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}
#[derive(Debug)]
#[derive(Clone)]
struct Client {
    name: String,
    ip_address: String,
    online: bool, // true for online, false for offline
    last_message_time: Instant, // Added field for last message time
}

impl Client {
    fn new(name: String, ip_address: String, online: bool) -> Client {
        Client {
            name,
            ip_address,
            online,
            last_message_time: Instant::now(), // Initialize with current time
        }
    }
}
//steganography 
fn steg_encode(path: String, message: String) -> String {
    let payload = str_to_bytes(&message);
    let destination_image = file_as_dynamic_image(path);
    let enc = Encoder::new(&payload, destination_image);
    let result = enc.encode_alpha();
    save_image_buffer(result, "default_with_hidden.png".to_string());
    let encoded_as_base64 = image_to_base64("default_with_hidden.png".to_string());
    encoded_as_base64
}
fn image_to_base64(path: String) -> String {
    let mut payload = File::open(path).unwrap();
    let mut payload_bytes = Vec::new();
    payload.read_to_end(&mut payload_bytes).unwrap();
    let res_base64 = base64::encode(payload_bytes);
    res_base64
 }
#[derive(Debug)]
#[derive(Clone)]
struct DirectoryOfService {
    clients: Vec<Client>,
}

impl DirectoryOfService {
    fn new() -> DirectoryOfService {
        DirectoryOfService {
            clients: Vec::new(),
        }
    }

    fn add_client(&mut self, name: String, ip_address: String, online: bool) {
        if let Some(client) = self.clients.iter_mut().find(|client| client.name == name && client.ip_address == ip_address) {
            // Client with the same name and IP address already exists, you can choose to update it or ignore the new client
            client.online = online;
        } else {
            // Client with the same name and IP address does not exist, add a new client
            self.clients.push(Client::new(name, ip_address, online));
        }
    }

    fn remove_offline_clients(&mut self) {
        let timeout_duration = Duration::new(20, 0);
        self.clients.retain(|client| client.online && client.last_message_time.elapsed() <= timeout_duration);
    }

    fn find_client_by_ip(&mut self, ip: &str) -> Option<&mut Client> {
        self.clients.iter_mut().find(|client| client.ip_address == ip)
    }
    fn find_client_by_name(&mut self, name: &str) -> Option<&mut Client> {
        self.clients.iter_mut().find(|client| client.name == name)
    }
    
}

lazy_static! {
    static ref DIRECTORY_OF_SERVICE: Mutex<DirectoryOfService> = Mutex::new(DirectoryOfService::new());
}

struct ServerData {
    local_address: Ipv4Addr,
    local_load: f32,  // A single f64 value for local CPU usage
    // other_servers: HashMap<Ipv4Addr, String>,  // A HashMap to store CPU usage values for other servers
    local_cpu_usage: f32,
    other_servers: Vec<(u32,Ipv4Addr, f32)>,
    is_leader: bool,
    local_id: u32,
    has_failure_token : bool,
    
    
}
// static mut COUNTER: u32 = 0;
static mut COUNTER: u32 = 0;
lazy_static! {
    static ref COUNTER_MUTEX: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
}
static SYS: Lazy<Arc<Mutex<System>>> = Lazy::new(|| Arc::new(Mutex::new(System::new_all())));

static mut server_data: ServerData = ServerData {
    local_address: Ipv4Addr::new(0, 0, 0, 0),
    local_load: 0.0,
    local_cpu_usage: 0.0,
    other_servers: Vec::new(),
    is_leader: false,
    local_id: 0,
    has_failure_token: false,
};

fn extract_ipv4_from_socket_addr(socket_addr: SocketAddr) -> Option<Ipv4Addr> {
    match socket_addr {
        SocketAddr::V4(v4) => Some(*v4.ip()),
        SocketAddr::V6(_) => None, // Handle IPv6 addresses as needed
    }
}

fn read_ipaddresses(path:String) -> Vec<Ipv4Addr> {
    let path = std::path::Path::new(&path);
    let mut servers = Vec::new();

    let contents = std::fs::read_to_string(path).expect("Something went wrong reading the file");
    for line in contents.lines() {
        let ipaddress: Ipv4Addr = line.parse().unwrap();
        servers.push(ipaddress);
    }

    return servers;
}

async fn send_unhandled_requests_to_client(requesting_client_name: String) -> Result<(), Box<dyn Error>> {
    let mut requests = REQUESTS.lock().unwrap();
    let servers = read_ipaddresses("servers.txt".to_string());
    let local_address = servers[1];
    let send_port = 0;
    let send_socket = UdpSocket::bind(SocketAddrV4::new(local_address, send_port)).await?;
    println!("Sending unhandled requests to client {}", requesting_client_name);
    let mut iter = requests.drain(..);
    while let Some(request) = iter.next() {
        if request.requesting_client_name == requesting_client_name {
            let owner_name = request.owning_client_name.clone();
            let mut directory = DIRECTORY_OF_SERVICE.lock().unwrap();

            let mut client_id = 0;
            let mut send_address = directory.find_client_by_name(&requesting_client_name).unwrap().ip_address.clone();

            if let Some(client) = directory.find_client_by_name(&owner_name) {
                // Simplified logic to map owning client name to client ID
                match request.owning_client_name.as_str() {
                    "Client1" => client_id = 1,
                    "Client2" => client_id = 2,
                    "Client3" => client_id = 3,
                    _ => {}
                }
            }

            drop(directory);

            let message = format!("{}:{}:{}", client_id, request.image_id, request.request);
            let mut send_to_client_port = 9095;
            if client_id == 1 {
                send_to_client_port = 9095;
            }
            else if client_id == 2 {
                send_to_client_port = 9094;
            }
            else if client_id == 3 {
                send_to_client_port = 9096;
            }

            // Add await here
            send_socket.send_to(message.as_bytes(), SocketAddrV4::new(send_address.parse().unwrap(), send_to_client_port)).await?;
            println!("Sent access rights message to client {}: {}", send_address, message);
        }
    }

    Ok(())
}


async fn receive_multicast_messages(message: u32) -> Result<(), Box<dyn Error>> {
    let servers= read_ipaddresses("servers.txt".to_string());
    let local_address= servers[1];
    let multicast_port = 8080;

    let socket = UdpSocket::bind(SocketAddrV4::new(local_address, multicast_port)).await?;

    let mut buffer = [0; 1024];

    println!("Listening for messages on {}:{}", local_address, multicast_port);

    loop {
        println!("Waiting for a message...");
        let (size, addr) = socket.recv_from(&mut buffer).await?;
        let received_message = String::from_utf8_lossy(&buffer[0..size]);
        if let Some(_ipv4) = extract_ipv4_from_socket_addr(addr) {

            let message_recieved = received_message.to_string();
                let mut split = message_recieved.split_whitespace();
                let id: &str = split.next().unwrap();
                let request_type = split.next().unwrap();
                let request = request_type.clone().to_string();
                let client_message = split.next().unwrap();
                let id = id.parse::<u32>().unwrap();
                let request_type = request_type.to_string();
                let mut client_message = client_message.to_string();  
               
            if request_type !="unhandled"{
                   
                let mut pending_requests = PENDING_REQUESTS.lock().unwrap();
                pending_requests.push(PendingRequest {
                    id,
                    address: extract_ipv4_from_socket_addr(addr).unwrap(),
                    request_type: request_type.clone(),
                    message: client_message,
                });
                drop (pending_requests);
                
                send_multicast_to_servers(message.clone()).await?;
            }
            else if request_type=="unhandled"
            {
                    let mut image_id = split.next().unwrap();
                    let mut access_rights = split.next().unwrap();
                    let mut requests = REQUESTS.lock().unwrap();
                    let mut request = Request {
                        requesting_client_name: String::new(),
                        owning_client_name: String::new(),
                        image_id: String::new(),
                        request: String::new(),
                    };
                    let mut owner_name = "Client".to_string() + &id.to_string();
                    request.add_request(client_message, owner_name, image_id.to_string(), access_rights.to_string());
                    requests.push(request);
                    drop(requests);
                }
            println!("Received response from client {}: {}", addr, received_message);
            {
                let mut directory = DIRECTORY_OF_SERVICE.lock().unwrap();

                if let Some(client) = directory.find_client_by_ip(&_ipv4.to_string()) {
                    let client_name = client.name.clone();
                    client.online = true;
                    client.last_message_time = Instant::now();
                } else {
                    let client_name: String = "Client".to_string()+ &id.to_string();
                    directory.add_client(client_name, _ipv4.to_string(), true);
                }
            }

        }      
    }
}

async fn encrypt_message(client_addr:Ipv4Addr) -> Result<(), Box<dyn Error>>{
    let servers= read_ipaddresses("servers.txt".to_string());
    let local_address= servers[1];
    // create a socket with any available port

    let rec_socket = UdpSocket::bind(SocketAddrV4::new(local_address, 0)).await?;
    let local_port = rec_socket.local_addr()?.port();

    // create another socket with any available port and send the port to the client
    let send_socket = UdpSocket::bind(SocketAddrV4::new(local_address, 0)).await?;
    let send_port = send_socket.local_addr()?.port();
    
    let message = format!("{} {}", local_port, "server2");
    println!("Sending message to client {}:{}", client_addr, send_port);
    send_socket.send_to(message.as_bytes(), SocketAddrV4::new(client_addr, 8082)).await?;
    // receive the message from the client
    let mut buffer = [0; 1024];

    println!("Listening for messages on {}:{}", local_address, local_port);
    
    loop {
        println!("Waiting for a message...");
        let mut received_chunks: Vec<Option<Vec<u8>>> = Vec::new();
            let mut total_chunks = 0;
            let mut id=0;
            let mut request_type = String::new();

        loop {
            let mut buf = [0; 2048]; // Adjust the buffer size as needed
            let (num_bytes, src_addr) = rec_socket.recv_from(&mut buf).await?;
    
            let message = String::from_utf8_lossy(&buf[..num_bytes]);
    
            // Parse the message into total_chunks, chunk_id, and chunk_data
            let mut parts: Vec<&str> = message.trim().split(':').collect();
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


                    let mut image_data: Vec<u8> = Vec::new();
                    for chunk in received_chunks {

                        if let Some(chunk) = chunk {
                            image_data.extend_from_slice(&chunk);
                        }
                        else {
                            eprintln!("Missing chunk");
                        }
                    }
                
                   // Save the reconstructed image
                   std::fs::write("received_image.png", &image_data)?;
                
                   println!("Image received successfully!");
                   let message = image_to_base64("received_image.png".to_string());

                   steg_encode("0001.png".to_string(), message);
                   // parse address to be without port
                   const MAX_CHUNK_SIZE: usize = 1024; 
                   let mut image_data = Vec::new();
                   let mut file = std::fs::File::open("default_with_hidden.png")?;

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
                       let message = format!("{}:{}:{}", total_chunks, chunk_id, base64_chunk);

                       task::block_on(async {
                           send_socket.send_to(message.as_bytes(), SocketAddrV4::new(client_addr, 8082)).await
                       })?;
                   
                       if chunk_id % 20 == 0 {
                           std::thread::sleep(std::time::Duration::from_millis(10));
                       }
                   }

                   println!("Image sent successfully!");

                    break;
                }

            } else 
            {
                eprintln!("Invalid message format: {}", message);
            }
        }
     }
}

  
async fn send_messages() -> Result<(), Box<dyn Error>> 
{
    let servers= read_ipaddresses("servers.txt".to_string());
    let local_address: Ipv4Addr= servers[1];
    let send_port = 8082;

    let send_socket = UdpSocket::bind(SocketAddrV4::new(local_address, send_port)).await?;

    let message = "Hello, from server 2!";
    let mut pending_requests = PENDING_REQUESTS.lock().unwrap();
    let mut pending_requests_clone = pending_requests.clone();
    if let Some(request) = pending_requests.get(0).cloned() {
        // Find the index of the request to remove
        if let Some(index) = pending_requests.iter().position(|a_request| a_request.id == request.id) 
        {
            let client_address = request.address;
            let request_type = request.request_type;
            
            if request_type== "encrypt"{
                let encrypt_handle = task::spawn(async move {
                    if let Err(err) = encrypt_message(client_address).await {
                        eprintln!("Error in sending unicast messages: {}", err);
                    }
                });
                pending_requests.remove(0);
                unsafe {
                    server_data.local_load -= 1.0;
                }
            }
            else if request_type=="request"{
                let mut query = request.message;
                // search for the client in the directory of service by name
                let mut directory = DIRECTORY_OF_SERVICE.lock().unwrap();
                if let Some(client) = directory.find_client_by_name(&query) {
                    // Client exists, send a unicast message to the client
                    let message = format!("{}", client.ip_address);
                    send_socket.send_to(message.as_bytes(), SocketAddrV4::new(client_address, send_port)).await?;
                    println!("Client found: Sent unicast message to client {}: {}", client_address, message);
                } else {
                    // Client does not exist, send a unicast message to the client
                    let message = format!("none");
                    send_socket.send_to(message.as_bytes(), SocketAddrV4::new(client_address, send_port)).await?;
                    println!("Client not found: Sent unicast message to client {}: {}", client_address, message);
                }
                pending_requests.remove(0);
                drop(directory);
                unsafe {
                    server_data.local_load -= 1.0;
                }
                
                
            } 
            else if request_type =="online"
            {
                let client_id =request.id.clone();
                let requesting_client_name = "Client".to_string() + &client_id.to_string();
                send_unhandled_requests_to_client(requesting_client_name).await?;
                pending_requests.remove(0);
                unsafe {
                    server_data.local_load -= 1.0;
                }
            }
        }
            
        drop (pending_requests); 
    }
    Ok(())
}
    


async fn send_failure_multicast_to_servers(recover : bool,share_token:bool)-> Result<(), Box<dyn Error>>
{
    let servers= read_ipaddresses("servers.txt".to_string());
    let local_address= servers[1];
    let failure_port = 8089;
    let failure_port_2 = 8087;

    let failure_socket = UdpSocket::bind(SocketAddrV4::new(local_address, failure_port)).await?;
    let failure_socket_2 = UdpSocket::bind(SocketAddrV4::new(local_address, failure_port_2)).await?;
    
    let mut message = "Server failed";
    if share_token == true {
        message = "take 1";
        unsafe{
            server_data.has_failure_token = true;
        }
    }
    else if recover == true {
        message = "Recover 1";
        unsafe{
            server_data.has_failure_token = false;
        }
    }
    else {
        unsafe{
            server_data.has_failure_token = true;
            
        }
        let mut i = 0;
        for server in &servers {
            if *server != local_address{
            i += 1;
            }
        else if *server == local_address{
            let path = std::path::Path::new("servers.txt");
            let contents = std::fs::read_to_string(path).expect("Something went wrong reading the file");
            let mut lines: Vec<&str> = contents.lines().collect();
            lines.remove(i);
            let mut new_contents = String::new();
            for line in lines {
                new_contents.push_str(line);
                new_contents.push_str("\n");
            }
            let new_path = std::path::Path::new("servers2.txt");
            std::fs::write(new_path, new_contents).expect("Something went wrong writing the file");
        
        }
    }
    }
    let mut buffer = [0; 1024];
    let mut i = 0;
    for server in &servers {
        if *server != local_address{
        failure_socket.send_to(message.as_bytes(), SocketAddrV4::new(*server, failure_port)).await?;
        failure_socket_2.send_to(message.as_bytes(), SocketAddrV4::new(*server, failure_port_2)).await?;
        i += 1;
        }
        
    }
   
    
    Ok(())

}

async fn send_multicast_to_servers(message:u32) -> Result<(), Box<dyn Error>> {

    let servers= read_ipaddresses("servers.txt".to_string());
    let local_address= servers[1];
    let multicast_to_server_port = 8084;
    let multicast_to_server_port_2 = 8085;
    

    let multicast_to_server_socket = UdpSocket::bind(SocketAddrV4::new(local_address, multicast_to_server_port)).await?;
    let multicast_to_server_socket_2 = UdpSocket::bind(SocketAddrV4::new(local_address, multicast_to_server_port_2)).await?;

    //message stands for the id and cpu usage
    //get current cpu usage
    //use SYS global variable
    let mut sys = SYS.lock().unwrap();
    sys.refresh_all();
    let mut cpu_usage = 0.0;
    for (pid, process) in sys.processes() {
        // Check if the process is running

            // Get CPU usage for each process
            let one_cpu_usage = process.cpu_usage();

            cpu_usage += one_cpu_usage;
    
    }
    // save the new cpu usage in server_data
    unsafe{
        server_data.local_cpu_usage = cpu_usage;
    }
    let message = format!("{:?} {:?}", message, cpu_usage);

    // let message = id. + usage.clone();  
    let mut buffer = [0; 1024];
    let mut i = 0;
    for server in &servers {
        if *server != local_address{
        println!("Sending message to server {}:{}", server, multicast_to_server_port);
        multicast_to_server_socket.send_to(message.as_bytes(), SocketAddrV4::new(*server, multicast_to_server_port)).await?;
        multicast_to_server_socket_2.send_to(message.as_bytes(), SocketAddrV4::new(*server, multicast_to_server_port_2)).await?;
        i += 1;
        
        }
        
    }
    
    
    Ok(())
}
fn elect_leader() -> Result<(bool), Box<dyn Error>> {
    println!("Election started");
    unsafe{
    let available_servers = read_ipaddresses("servers2.txt".to_string());
    let local_usage= server_data.local_cpu_usage;
    let mut min_usage = local_usage;
    let mut min_server = server_data.local_address;
    //print local usage
    for (id,server, usage) in &server_data.other_servers {
        //check if server is available
        if !available_servers.contains(server) {
            println!("Server {:?} is not available", server);
            continue;
        }
        let usage = usage;
        
        if usage < &local_usage {
           
            min_usage = *usage;
            min_server = *server;
        }
        else if *usage == min_usage {
    
            if id < &server_data.local_id {

                min_server = *server;
               
            }
            else {
                min_server = server_data.local_address;
            }
        }
        println!("min server is {:?}", min_server);
    }
    if min_server == server_data.local_address {
            server_data.is_leader = true;
            server_data.local_load += 1.0;
            let unicast_handle = thread::spawn(|| {
            if let Err(err) = async_std::task::block_on(send_messages()) {
                eprintln!("Error in sending unicast messages: {}", err);
            }
            server_data.is_leader = false;
        });
        unicast_handle.join().expect("Unicast thread panicked");
        println!("I am the leader");
    }
    else {
        println!("I am not the leader so request is not processed");
        let mut pending_requests = PENDING_REQUESTS.lock().unwrap();
        let mut i = 0;
        
        PendingRequest::remove_request_by_index(&mut pending_requests, i);
        let mut last_request = pending_requests.pop();
        if let Some(pending_request) = last_request {
            if pending_request.request_type == "online" {
                let mut id = pending_request.id;
                let mut client_name = "Client".to_string() + &id.to_string();
                let mut requests = REQUESTS.lock().unwrap();
                requests.retain(|request| request.requesting_client_name != client_name);
                drop (requests);
            }
        }
        
        drop(pending_requests);
    }
    println!("leader is {:?}", min_server);
 
    return Ok(server_data.is_leader);
}

}


async fn receive_failure_from_server1()-> Result<(), Box<dyn Error>>
{
    let servers= read_ipaddresses("servers.txt".to_string());
    let local_address= servers[1];
    let failure_port = 8088;

    let failure_socket = UdpSocket::bind(SocketAddrV4::new(local_address, failure_port)).await?;
    // let failure_socket_2 = UdpSocket::bind(SocketAddrV4::new(local_address, failure_port_2)).await?;

    let mut buffer = [0; 1024];
    loop{

    let (size , addr) = failure_socket.recv_from(&mut buffer).await?;
        let received_message = String::from_utf8_lossy(&buffer[0..size]);
        // println!("Received failure message from {}: {}", addr, received_message);
        if let Some(ipv4) = extract_ipv4_from_socket_addr(addr) {
            //check if recieved message is "Recover"if it is then recover from failure if it is not then erase from servers.txt
            let message_recieved = received_message.to_string();
            let mut split = message_recieved.split_whitespace();
            let title: &str = split.next().unwrap();
            let server_id = split.next().unwrap();
            if title == "Recover" {
                recover_from_failure(server_id.parse::<usize>().unwrap());
                if server_id.parse::<usize>().unwrap() == 0 {
                    unsafe{
                        server_data.has_failure_token = false;
                    }

                }
                // send directory of service to server 3
                let mut directory = DIRECTORY_OF_SERVICE.lock().unwrap();
                let mut message = String::new();
                for client in &directory.clients {
                    message.push_str(&client.name);
                    message.push_str(" ");
                    message.push_str(&client.ip_address);
                    message.push_str(" ");
                    message.push_str(&client.online.to_string());
                    message.push_str(" : ");
                }
                message.pop();
                let message = format!("{} {}", "directory", message);
                let send_port = 8092;
                let send_socket = UdpSocket::bind(SocketAddrV4::new(local_address, send_port)).await?;
                send_socket.send_to(message.as_bytes(), SocketAddrV4::new(ipv4, send_port)).await?;

            }
            else if title=="take" 
            {
                // println!("recieved from server 1 i am not taking token");
                unsafe{
                    server_data.has_failure_token = false;
                }
            }
            
            else if title=="Server"
            {
                unsafe{
                    server_data.has_failure_token = false;
                }
            
                let mut i = 0;
                for server in &servers {
                    if *server == ipv4 {
                        // erase from servers.txt
                        let path = std::path::Path::new("servers.txt");
                        let contents = std::fs::read_to_string(path).expect("Something went wrong reading the file");
                        let mut lines: Vec<&str> = contents.lines().collect();
                        lines.remove(i);
                        let mut new_contents = String::new();
                        for line in lines {
                            new_contents.push_str(line);
                            new_contents.push_str("\n");
                        }
                        let new_path = std::path::Path::new("servers2.txt");
                        std::fs::write(new_path, new_contents).expect("Something went wrong writing the file");
                        // servers.remove(i);
                        // println!("Server failed: {:?}", ipv4);
                        // break;
                    }
                    i += 1;
                
                    }
                }
    }
    }
   
}
async fn receive_failure_from_server3() -> Result<(), Box<dyn Error>>
{
    let servers= read_ipaddresses("servers.txt".to_string());
    let local_address= servers[1];
   
    let failure_port_2 = 8090;

    //let failure_socket = UdpSocket::bind(SocketAddrV4::new(local_address, failure_port)).await?;
    let failure_socket_2 = UdpSocket::bind(SocketAddrV4::new(local_address, failure_port_2)).await?;

    let mut buffer = [0; 1024];
    loop{

        let (size , addr) = failure_socket_2.recv_from(&mut buffer).await?;
        let received_message = String::from_utf8_lossy(&buffer[0..size]);
        //println!("Received failure message from {}: {}", addr, received_message);
        if let Some(ipv4) = extract_ipv4_from_socket_addr(addr) {
            // println!("Server failed IN FIRST IF: {:?}", ipv4);
            let message_recieved = received_message.to_string();
            let mut split = message_recieved.split_whitespace();
            let title: &str = split.next().unwrap();
            let server_id = split.next().unwrap();
            if title == "Recover" {
                recover_from_failure(server_id.parse::<usize>().unwrap());
                if server_id.parse::<usize>().unwrap() == 0 {
                    unsafe{
                        server_data.has_failure_token = false;
                    }

                }
                else if server_id.parse::<usize>().unwrap() == 2 {
                    unsafe{
                        server_data.has_failure_token = true;
                    }
                } 

            }
            else if title=="take" 
            {
                // println!("server 3 did not fail and taking token");
                unsafe{
                    server_data.has_failure_token = true;
                }
            }

            if title=="Server"
            {
                unsafe{
                    server_data.has_failure_token = false;
                }
                let mut i = 0;
                for server in &servers {
                if *server == ipv4 {
                    // erase from servers.txt
                    let path = std::path::Path::new("servers.txt");
                    let contents = std::fs::read_to_string(path).expect("Something went wrong reading the file");
                    let mut lines: Vec<&str> = contents.lines().collect();
                    lines.remove(i);
                    let mut new_contents = String::new();
                    for line in lines {
                        new_contents.push_str(line);
                        new_contents.push_str("\n");
                    }
                    let new_path = std::path::Path::new("servers2.txt");
                    std::fs::write(new_path, new_contents).expect("Something went wrong writing the file");
                    // servers.remove(i);
                    // println!("Server failed: {:?}", ipv4);
                    // break;
                }
                i += 1;
            }
            }

           
        }
    }


   
}

fn recover_from_failure(index : usize){
    println!("Recovering from failure");
    let servers = read_ipaddresses("servers.txt".to_string());
    let local_address: Ipv4Addr = servers[index];
    // add local address to correct index in servers2.txt
    let path = std::path::Path::new("servers2.txt");
    let contents = std::fs::read_to_string(path).expect("Something went wrong reading the file");
    let mut lines: Vec<&str> = contents.lines().collect();

    // check the index of local address in servers.txt and add it to the same index in servers2.txt
    let mut i = 0;
    let address = local_address.to_string();
    for server in &servers {
        if *server == local_address {
            lines.insert(i, &address);
            break;
        }
        i += 1;
    }
    // println!("lines is {:?}", lines);
    let mut new_contents = String::new();
    for line in lines {
        new_contents.push_str(line);
        new_contents.push_str("\n");
    }
    unsafe{
        server_data.has_failure_token = false;
    }
    // println!("new contents is {:?}", new_contents);
    std::fs::write(path, new_contents).expect("Something went wrong writing the file");

}
async fn failure (load:f32) -> Result<(), Box<dyn Error>> {
    let interval = Duration::new(20, 0);
    let mut last_send_time = Instant::now();
    let mut recover = true;
    let mut load = load.clone();
        loop {
            unsafe{
                if server_data.has_failure_token == true {
                    let now = Instant::now();
                    if now.duration_since(last_send_time) >= interval {
                        recover = !recover;
                        // println!("inside loop");
                        unsafe{
                            load = server_data.local_load;
                        }
                        if load== 0.0 {
                            // println!("inside if");
                    
                        if let Err(err) = async_std::task::block_on(send_failure_multicast_to_servers(recover,false)) {
                            eprintln!("Error in sending multicast messages: {}", err);
                        }
                        if recover ==true{
                            // println!("recovering");
                            recover_from_failure(1);
                            if let Err(err) = async_std::task::block_on(update_directory_of_service()) {
                                eprintln!("Error in receiving directory of service: {}", err);
                            }
                           // println!("updated directory of service after recovering");
                            let mut directory = DIRECTORY_OF_SERVICE.lock().unwrap();
                            for client in &directory.clients {
                                println!("Client Name: {:?}, Client IP Address: {:?}, Client Online: {:?}", client.name, client.ip_address, client.online);
                            }

                        }
                        last_send_time = now;
                     }
                     if load != 0.0 {
                        // println!("load is not 0 but failing now ");
                    
                        if let Err(err) = async_std::task::block_on(send_failure_multicast_to_servers(recover,true)) {
                            eprintln!("Error in sending multicast messages: {}", err);
                        }
        
                     }
                    }
        
                    thread::sleep(Duration::from_secs(30));
                }
            }
           
        }
    
    }
    
async fn receive_multicast_from_server1() -> Result<(), Box<dyn Error>> {
    let mut servers = read_ipaddresses("servers.txt".to_string());
    let local_address = servers[1];
    let multicast_from_server_port = 8083;
  
    

    let multicast_from_server_socket = UdpSocket::bind(SocketAddrV4::new(local_address, multicast_from_server_port)).await?;
    

    let mut buffer = [0; 1024];


    println!("Listening for messages on {}:{}", local_address, multicast_from_server_port);


   

    loop {
        println!("Waiting for a message...");
        let (size, addr) = multicast_from_server_socket.recv_from(&mut buffer).await?;
        let received_message = String::from_utf8_lossy(&buffer[0..size]);
        println!("Received response from server {}: {}", addr, received_message);
        
        // Extract the IPv4 address from the SocketAddr
        if let Some(ipv4) = extract_ipv4_from_socket_addr(addr) {

            let mut counter = COUNTER_MUTEX.lock().unwrap();
                *counter += 1;
        
            let message_recieved = received_message.to_string();
            // PARSE message by space the first part is the id and the second is the cpu usage
            let mut split = message_recieved.split_whitespace();
            let id = split.next().unwrap();
            let local_load = split.next().unwrap();
            //convert id to u32
            let id = id.parse::<u32>().unwrap();
            let local_load = local_load.parse::<f32>().unwrap();
           
    
            unsafe {
                let server_exists = server_data
                .other_servers
                .iter()
                .any(|(_, addr, _)| addr == &ipv4);
                if !server_exists {
                    server_data.other_servers.push((id, ipv4, local_load));
                }
                else {
                    for (id, server, usage) in &mut server_data.other_servers {
                        if server == &ipv4 {
                            *usage = local_load;
                        }
                    }
                }
        
                println!("Local Address: {:?}", server_data.local_address);
                println!("CPU Usage for Local Server: {:?}", server_data.local_cpu_usage);
            
                for (id, server, usage) in &server_data.other_servers {
                    println!("Server ID: {:?}, Server Address: {:?}, CPU Usage: {:?}", id, server, usage);
                }
            }
        
            
        }
        
    }
    
}
async fn receive_multicast_from_server3() -> Result<(), Box<dyn Error>> {
    let servers = read_ipaddresses("servers.txt".to_string());
    let local_address = servers[1];
    let multicast_from_server_port_2 = 8086;
    

    let multicast_from_server_socket_2 = UdpSocket::bind(SocketAddrV4::new(local_address, multicast_from_server_port_2)).await?;

    let mut buffer = [0; 1024];
  

    loop {
        
        let (size, addr) = multicast_from_server_socket_2.recv_from(&mut buffer).await?;
        let received_message = String::from_utf8_lossy(&buffer[0..size]);
        println!("Received response from server {}: {}", addr, received_message);

        if let Some(ipv4) = extract_ipv4_from_socket_addr(addr) {
            let message_recieved = received_message.to_string();
            let mut split = message_recieved.split_whitespace();
            let id = split.next().unwrap();
            let local_load = split.next().unwrap();
            let id = id.parse::<u32>().unwrap();
            let local_load = local_load.parse::<f32>().unwrap();
            let mut counter = COUNTER_MUTEX.lock().unwrap();
            *counter += 1;
           unsafe{ 
                let server_exists = server_data
                .other_servers
                .iter()
                .any(|(_, addr, _)| addr == &ipv4);
                if !server_exists {
                    server_data.other_servers.push((id, ipv4, local_load));
                }else {
                    for (id, server, usage) in &mut server_data.other_servers {
                        if server == &ipv4 {
                            *usage = local_load;
                        }
                    }
                }
            }
        }
        
    }
        
}
async fn check_counter() -> Result<(), Box<dyn Error>> {
    loop{
        let servers = read_ipaddresses("servers.txt".to_string());
        let servers2= read_ipaddresses("servers2.txt".to_string());
        let mut _size = servers2.len();
        let mut counter = COUNTER_MUTEX.lock().unwrap();
        let found = servers2.iter().any(|&i| i == servers[1]);
        if *counter == 2 && found == true {
            thread::sleep(Duration::from_secs(2));
            elect_leader();
            *counter = 0;
        }
        else if *counter == 2 && found == false {
            *counter = 0;
        }

    }
}
async fn update_directory_of_service()-> Result<(), Box<dyn Error>> {

    let servers= read_ipaddresses("servers.txt".to_string());
    let local_address= servers[1];
   
    let directory_port = 8093;
    let directory_socket = UdpSocket::bind(SocketAddrV4::new(local_address, directory_port)).await?;

    let mut buffer = [0; 1024];
    loop{
        let (size , addr) = directory_socket.recv_from(&mut buffer).await?;
        let received_message = String::from_utf8_lossy(&buffer[0..size]);
        println!("Received directory of service from {}: {}", addr, received_message);
        if let Some(ipv4) = extract_ipv4_from_socket_addr(addr) {
            let message_recieved = received_message.to_string();
            let parts: Vec<&str> = message_recieved.split(':').collect();
            for part in parts {
                let mut split = part.split_whitespace();
                let client_name = split.next().unwrap();
                let client_ip = split.next().unwrap();
                let client_online = split.next().unwrap();
                
                let client_name = client_name.to_string();
                let client_ip = client_ip.to_string();
                let client_online = client_online.parse::<bool>().unwrap();
                let mut directory = DIRECTORY_OF_SERVICE.lock().unwrap();

                if let Some(client) = directory.find_client_by_ip(&client_ip) {
                    client.online = client_online;
                } else {
                    directory.add_client(client_name, client_ip, client_online);
                }
            }
            
        }
    }   
}
fn main() -> Result<(), Box<dyn Error>> {

 
    let mut cpu_usage = 0.0;
    

    let servers= read_ipaddresses("servers.txt".to_string());

    unsafe {
        server_data.local_address = servers[1];
        server_data.local_load = 1.0;
        server_data.local_cpu_usage = cpu_usage;
        server_data.other_servers = Vec::new();
        server_data.is_leader = false;
        server_data.local_id = 2;
        server_data.has_failure_token = false;
    }

  
    let usage;
    let local_id;


    unsafe {
        usage = server_data.local_load.clone();
        local_id = server_data.local_id;
    }

    // let message = format!("{:?} {:?}", local_id, usage);
    let message = local_id;
    let receive_multicast_from_server_handle = thread::spawn(move || {
            if let Err(err) = async_std::task::block_on(receive_multicast_from_server1()) {
            eprintln!("Error in receiving multicast messages from servers: {}", err);
    }
    });
    let receive_multicast_from_server_handle_2 = thread::spawn(move || {
        if let Err(err) = async_std::task::block_on(receive_multicast_from_server3()) {
        eprintln!("Error in receiving multicast messages from servers: {}", err);
}
});
    
    let multicast_handle = thread::spawn(move || {
        if let Err(err) = async_std::task::block_on(receive_multicast_messages(message)) {
            eprintln!("Error in receiving multicast messages: {}", err);
        }
    });
    
    let failure_handle = thread::spawn(move || {
        if let Err(err) = async_std::task::block_on(failure(usage)) {
            eprintln!("Error in receiving handling failure messages: {}", err);
        }
    });
    let check_counter_handle = thread::spawn(move || {
        if let Err(err) = async_std::task::block_on(check_counter()) {
            eprintln!("Error in receiving handling failure messages: {}", err);
        }
    });

    let failure_handle_1 = thread::spawn(move || {

        if let Err(err) = async_std::task::block_on(receive_failure_from_server1()) {
            eprintln!("Error in recieving failure from server1: {}", err);
        }
    });
    let failure_handle_2 = thread::spawn(move || {
        if let Err(err) = async_std::task::block_on(receive_failure_from_server3()) {
            eprintln!("Error in recieving failure from server3: {}", err);
        }
    });
    


    multicast_handle.join().expect("Multicast from client thread panicked");
    receive_multicast_from_server_handle.join().expect("Multicast from server 1 thread panicked");
    failure_handle.join().expect("Multicast thread panicked");
    receive_multicast_from_server_handle_2.join().expect("Multicast from server 2 thread panicked");
    check_counter_handle.join().expect("counter thread panicked");
    
    failure_handle_1.join().expect("failure 1 thread panicked");
    failure_handle_2.join().expect("failure 2 thread panicked");


    Ok(())

}