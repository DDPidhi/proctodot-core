use tokio::sync::mpsc::UnboundedSender;
use crate::enums::user_type::UserType;

pub struct Participant {
    pub sender: UnboundedSender<String>,
    pub id: i32,
    pub connection_id: String,
}

pub struct WebSocketServer {
    proctor: Option<UnboundedSender<String>>,
    proctor_id: i32,
    proctor_connection_id: String,
    participants: Vec<Participant>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        Self {
            proctor: None,
            proctor_id: 0,
            proctor_connection_id: "".to_string(),
            participants: Vec::new(),
        }
    }

    /// Registers a user (proctor or participant) with the WebSocketServer
    pub fn register(
        &mut self,
        user_type: String,
        sender: UnboundedSender<String>,
        user_id: i32,
        connection_id: String,
    ) {
        if user_type == UserType::Member.to_string() {
            self.participants.push(Participant {
                sender,
                id: user_id,
                connection_id: connection_id.clone(),
            });
            println!(
                "✅ Participant registered with ID: {}, Connection ID: {}",
                user_id, connection_id
            );
        } else if user_type == UserType::Proctor.to_string() {
            self.proctor = Some(sender);
            self.proctor_id = user_id;
            self.proctor_connection_id = connection_id;
            println!(
                "✅ Proctor registered with ID: {}, Connection ID: {}",
                self.proctor_id, self.proctor_connection_id
            );
        } else {
            println!("❌ Invalid user type: {}", user_type);
        }
    }

    /// Sends a message to the appropriate recipient based on the recipient ID
    pub fn send_to(&self, sender_id: i32, recipient_id: i32, message: String, event: String) {
        let json_payload = serde_json::json!({
            "event": event,
            "message": message,
            "sender_id": sender_id.to_string(),
        });

        if recipient_id == self.proctor_id {
            if let Some(proctor_sender) = &self.proctor {
                println!("📤 Sending message to proctor with ID: {}, sender: {}", self.proctor_id, sender_id);
                if proctor_sender.send(json_payload.to_string()).is_err() {
                    println!("❌ Failed to send message to proctor: Channel closed");
                }
            } else {
                println!("❌ No proctor connected");
            }
        } else if let Some(participant) = self.participants.iter().find(|p| p.id == recipient_id) {
            println!("📤 Sending message to participant with ID: {}", participant.id);
            if participant.sender.send(json_payload.to_string()).is_err() {
                println!("❌ Failed to send message to participant: Channel closed");
            }
        } else {
            println!("❌ Recipient ID not recognized: {}", recipient_id);
        }
    }

    /// Unregisters a user when their WebSocket connection closes
    pub fn unregister(&mut self, connection_id: String) {
        if connection_id == self.proctor_connection_id {
            println!("🔴 Unregistering proctor with ID: {}", self.proctor_id);
            self.proctor = None;
            self.proctor_id = 0;
            self.proctor_connection_id.clear();
        } else {
            if let Some(index) = self.participants.iter().position(|p| p.connection_id == connection_id) {
                println!("🔴 Unregistering participant with ID: {}", self.participants[index].id);
                self.participants.remove(index);
            } else {
                println!("❌ Connection ID not recognized: {}", connection_id);
            }
        }
    }

    /// Retrieves the proctor's ID
    pub fn get_proctor_id(&self) -> i32 {
        self.proctor_id
    }

    /// Retrieves the participant ID by connection ID
    pub fn get_participant_id(&self, connection_id: String) -> Option<i32> {
        self.participants
            .iter()
            .find(|p| p.connection_id == connection_id)
            .map(|p| p.id)
    }

}
