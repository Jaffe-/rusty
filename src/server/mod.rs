use std::net::{TcpStream, TcpListener};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::thread;
use std::io::{Read, Write};
use std::collections::HashMap;

#[derive(Clone)]
enum Event {
    NewConnection(String, Sender<Event>),
    Network(NetworkEvent),
    LostConnection(String)
}

#[derive(Clone)]
enum NetworkEvent {
    NewClient(String),
    NewMessage(String),
    LostClient(String)
}

trait Module {
    fn get_sender(&self) -> Sender<Event>;
    fn run(&mut self);
    fn notify(&mut self, event: Event) {
        self.get_sender().send(event);
    }
}

struct Client {
    stream: TcpStream,
    nick: String,
    tx: Sender<Event>,
    rx: Receiver<Event>,
    this_tx: Sender<Event>
}

impl Client {
    fn new(stream: TcpStream, tx: Sender<Event>) -> Client {
        let (this_tx, this_rx) = channel();
        Client { stream: stream, nick: String::new(), tx: tx,
                 rx: this_rx, this_tx: this_tx }
    }

    fn handle_events(&mut self) {
        match self.rx.try_recv() {
            Ok(Event::Network(NetworkEvent::NewClient(s))) => {
                write!(self.stream, "HAHAHAH\n");
            }
            Ok(..) => {
                println!("Some other event");
            },
            Err(..) => {
            }
        }
    }

    fn whatever_else(&self) {
    }
}

impl Module for Client {
    fn get_sender(&self) -> Sender<Event> {
        self.this_tx.clone()
    }

    fn run(&mut self) {
        loop {
            self.handle_events();
            self.whatever_else();
        }
    }
}

struct Dispatcher {
    txs: HashMap<String, Sender<Event>>,
    rx: Receiver<Event>,
    tx: Sender<Event>
}

impl Dispatcher {
    fn new() -> Dispatcher {
        let (this_tx, this_rx) = channel();
        Dispatcher { txs: HashMap::new(), rx: this_rx, tx: this_tx }
    }

    fn add_node(&mut self, id: String, tx: Sender<Event>) {
        self.txs.insert(id, tx);
    }

    fn run(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(event) => {
                    match event {
                        Event::NewConnection(addr, sender) => {
                            println!("Adding new node {}", addr);
                            for (id, tx) in &mut self.txs {
                                tx.send(Event::Network(NetworkEvent::NewClient(addr.to_string())));
                            }
                            self.add_node(addr, sender);
                        }
                        _ => {
                            //for (id, tx) in &mut self.txs {
                            //    tx.send(event.clone());
                            //}
                        }
                    }
                }
                Err (..) => {}
            }
        }
    }

    fn get_sender(&self) -> Sender<Event> {
        self.tx.clone()
    }
}

struct Server {
    listener: TcpListener,
    rx: Receiver<Event>,
    tx: Sender<Event>
}

impl Server {
    fn new(ip: &String, port: &String, tx: Sender<Event>) -> Result<Server, &'static str> {
        let (this_tx, this_rx) = channel();
        match TcpListener::bind(format!("{}:{}", ip, port).as_str()) {
            Ok(listener) => {
                Ok(Server { listener: listener, rx: this_rx, tx: tx })
            }
            Err(..) => Err("Unable to bind to the given interface")
        }
    }

    fn listen(&mut self) {
        loop {
            match self.listener.accept() {
                Ok((stream, socket_addr)) => {
                    println!("Client at IP {} connected", socket_addr);
                    let mut client = Client::new(stream, self.tx.clone());
                    self.tx.send(Event::NewConnection("bogus".to_string(), client.get_sender()));
                    thread::spawn(move || {
                        client.run();
                    });
                }
                Err(..) => {}
            }
        }
    }

    fn get_sender(&self) -> Sender<Event> {
        self.tx.clone()
    }
}

pub fn run_server(ip: &String, port: &String) {
    let mut dispatcher = Dispatcher::new();
    match Server::new(ip, port, dispatcher.get_sender()) {
        Ok(mut server) => {
            dispatcher.add_node("".to_string(), server.get_sender());
            thread::spawn(move || {
                server.listen();
            });

            dispatcher.run();
        }
        Err(err) => {
            println!("Failed: {}", err);
        }
    }

}
