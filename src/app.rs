use chrono::{DateTime, Local};
use ratatui::widgets::{List, ListState};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub enum CurrentScreen {
    Connecting,
    Chat,
    Quiting,
}

#[derive(Clone)]
pub enum Sender {
    Local,
    Remote,
}

pub enum ConnectEditing {
    Address,
    Name,
}

#[derive(Clone)]
pub struct Message {
    pub sender: Sender,
    pub sender_name: String,
    pub text: String,
    pub time: DateTime<Local>,
}
pub struct App {
    pub current_screen: CurrentScreen,
    pub address: String,
    pub name: String,
    pub messages: Arc<Mutex<Vec<Message>>>,
    pub editing_text: String,
    pub connect_editing: ConnectEditing,
    msg_tx: mpsc::Sender<Message>,
    msg_rx: Arc<Mutex<Receiver<Message>>>,
    pub list_state: ListState,
}

impl App {
    pub fn new() -> App {
        let (msg_tx, msg_rx) = mpsc::channel();
        App {
            current_screen: CurrentScreen::Connecting,
            address: String::new(),
            name: String::new(),
            messages: Arc::new(Mutex::new(Vec::new())),
            editing_text: String::new(),
            connect_editing: ConnectEditing::Address,
            msg_tx,
            msg_rx: Arc::new(Mutex::new(msg_rx)),
            list_state: ListState::default(),
        }
    }

    pub fn send_message(&mut self) {
        // 如果发送的消息为空（全是空格）则不发送
        if self.editing_text.trim().len() == 0 {
            return;
        }
        let message_text = self.editing_text.clone().trim().to_string();
        let message = Message {
            sender: Sender::Local,
            sender_name: self.name.clone(),
            text: message_text.clone(),
            time: Local::now(),
        };
        self.messages.lock().unwrap().push(message.clone());

        self.msg_tx.clone().send(message).unwrap();
        self.editing_text.clear();
    }

    pub fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.address = self.address.trim().to_string();
        self.name = self.name.trim().to_string();
        self.current_screen = CurrentScreen::Chat;
        let mut stream = TcpStream::connect(&self.address)?;
        // 发送线程
        let mut sender_stream = stream.try_clone()?;
        let rx = self.msg_rx.clone();
        thread::spawn(move || {
            let rx = rx.lock().unwrap();
            for msg in rx.iter() {
                let mut sender_name = msg.sender_name.clone();
                // 确保 sender_name 是恰好 32 字节
                sender_name.truncate(32); // 如果超过 32 字节则截断
                if sender_name.len() < 32 {
                    sender_name.extend(std::iter::repeat(' ').take(32 - sender_name.len()));
                    // 不足 32 字节用空格填充
                }
                //
                let mut buffer = Vec::new();
                buffer.extend_from_slice(sender_name.as_bytes());
                buffer.extend_from_slice(msg.text.as_bytes());

                sender_stream
                    .write_all(&buffer)
                    .expect("failed to write stream");
                sender_stream.flush().expect("failed to flush stream");
            }
        });
        // 监听 socket 消息接收线程
        let messages = self.messages.clone();
        thread::spawn(move || {
            loop {
                let mut buf = [0; 1024];
                match stream.read(&mut buf) {
                    Ok(0) => break, // 连接已关闭
                    Ok(len) => {
                        let msg = Message {
                            sender: Sender::Remote,
                            sender_name: String::from_utf8_lossy(&buf[..32]).trim().to_string(),
                            text: String::from_utf8_lossy(&buf[32..len]).to_string(),
                            time: Local::now(),
                        };
                        messages.lock().unwrap().push(msg);
                    }
                    Err(e) => {
                        panic!("failed to read stream: {}", e);
                    }
                }
            }
        });
        Ok(())
    }

    pub fn quit(&mut self) {
        self.current_screen = CurrentScreen::Quiting;
    }
}
