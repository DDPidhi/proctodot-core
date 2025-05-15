use crate::http::socket::websocket_server::WebSocketServer;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use tokio::sync::mpsc;
use serde_json::Value;
use sea_orm::{DatabaseConnection, EntityTrait, Set};
use std::sync::{Arc, Mutex};
use actix::{Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws;
use uuid::Uuid;
use actix::fut::wrap_future;
use sea_orm::QueryFilter;
use crate::http::controllers::base_controller::{BaseController, Controller};
use sea_orm::ColumnTrait;
use std::collections::HashMap;
use lazy_static::lazy_static;
use actix::Message as ActixMessage;
use crate::entities::{chat_rooms, users};
use crate::enums::user_type::UserType;

lazy_static! {
    static ref WS_SERVERS: Arc<Mutex<HashMap<String, Arc<Mutex<WebSocketServer>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Custom message type for WebSocket text messages
#[derive(Debug, ActixMessage)]
#[rtype(result = "()")]
pub struct TextMessage(pub String, pub String, pub String);

pub struct WebSocketActor {
    ws_server: Arc<Mutex<WebSocketServer>>,
    connection_id: String,
    room_id: String,
    user_type: String,
    user_id: i32,
    db_pool: web::Data<DatabaseConnection>,
}

impl Actor for WebSocketActor {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!(
            "🔗 WebSocket connection started for connection_id: {}, user_type: {}, user_id: {}",
            self.connection_id, self.user_type, self.user_id
        );

        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let mut ws_server = self.ws_server.lock().unwrap();

        // Register the user as soon as the connection starts
        ws_server.register(
            self.user_type.clone(),
            tx,
            self.user_id.clone(),
            self.connection_id.clone(),
        );

        // Use wrap_future to convert the async block into an ActorFuture
        let addr = ctx.address();
        ctx.spawn(wrap_future(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(json) = serde_json::from_str::<Value>(&msg.clone()) {
                    if let (Some(event), Some(message), Some(senderId)) = (json.get("event"), json.get("message"), json.get("sender_id")) {
                        println!(
                            "📨 Received message to send to WebSocket: {} and Event: {}",
                            message,
                            event
                        );
                        addr.do_send(
                            TextMessage(
                                message.as_str().unwrap_or("").to_string(),
                                event.as_str().unwrap_or("").to_string(),
                                senderId.as_str().unwrap_or("").to_string(),
                            )
                        );
                    }
                }
            }
        }));
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        println!("❌ Disconnecting {}", self.connection_id);
        let mut ws_server = self.ws_server.lock().unwrap();
        ws_server.unregister(self.connection_id.clone());
        actix::Running::Stop
    }
}

impl WebSocketActor {
    fn new(
        ws_server: Arc<Mutex<WebSocketServer>>,
        connection_id: String,
        room_id: String,
        user_type: String,
        user_id: i32,
        db_pool: web::Data<DatabaseConnection>,
    ) -> Self {
        WebSocketActor {
            ws_server,
            connection_id,
            room_id,
            user_type,
            user_id,
            db_pool,
        }
    }
}

impl Handler<TextMessage> for WebSocketActor {
    type Result = ();

    fn handle(&mut self, msg: TextMessage, ctx: &mut Self::Context) {
        println!("📥 Received TextMessage for WebSocketActor: {:?}", msg);
        let json_payload = serde_json::json!({
            "event": msg.1,
            "message": msg.0,
            "sender_id": msg.2,
        });
        ctx.text(json_payload.to_string());
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for WebSocketActor {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Message::Text(text)) => {
                println!("📩 Message from {}: {}", self.connection_id, self.user_type);

                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    if let (Some(event), Some(message), Some(participant)) = (json.get("event"), json.get("message"), json.get("participant").and_then(|v| v.as_i64())) {
                        let ws_server = self.ws_server.lock().unwrap();
                        let recipient_id = if self.user_type == UserType::Member.to_string() {
                            ws_server.get_proctor_id()
                        } else {
                            participant as i32
                        };

                        println!(
                            "📨 Forwarding message from user_id: {} to recipient_id: {}",
                            self.user_id, recipient_id
                        );

                        ws_server.send_to(
                            self.user_id,
                            recipient_id,
                            message.as_str().unwrap_or("").to_string(),
                            event.as_str().unwrap_or("").to_string(),
                        );
                    }
                }
            }
            Ok(Message::Ping(ping)) => {
                ctx.pong(&ping);
            }
            Ok(Message::Close(_)) => {
                println!("❌ Connection closed for {}", self.connection_id);
                self.ws_server
                    .lock()
                    .unwrap()
                    .unregister(self.connection_id.clone());
                ctx.stop();
            }
            _ => (),
        }
    }
}

async fn get_existing_chat_room(room_id: &str, db: &DatabaseConnection) -> Option<String> {
    let room = chat_rooms::Entity::find()
        .filter(chat_rooms::Column::RoomId.eq(room_id))
        .one(db)
        .await
        .ok()?;
    room.map(|r| r.room_id)
}

async fn save_chat_room(room_id: &str, db: &DatabaseConnection) {
    let room = chat_rooms::ActiveModel {
        room_id: Set(room_id.to_string()),
        ..Default::default()
    };
    chat_rooms::Entity::insert(room).exec(db).await.unwrap();
}

pub async fn websocket_index(
    req: HttpRequest,
    stream: web::Payload,
    db_pool: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let room_id = req.match_info().get("room_id").unwrap_or("unknown").to_string();

    let claims = Controller::get_claims(&req);
    if claims.is_none() {
        return Ok(Controller::unauthorized("Unauthorized"));
    }

    let user_id: i32 = claims.unwrap().sub.parse().unwrap();
    let user = users::Entity::find()
        .filter(users::Column::Id.eq(user_id))
        .one(db_pool.get_ref())
        .await
        .ok();
    println!(
        "🌐 WebSocket connection for room ID: {}, requester ID: {}",
        room_id, user_id
    );

    let room_id = match get_existing_chat_room(&room_id, &db_pool).await {
        Some(existing_room_id) => existing_room_id,
        None => {
            save_chat_room(&room_id, &db_pool).await;
            room_id
        }
    };

    println!("🏠 Room ID for this session: {}", room_id);

    let ws_server = {
        let mut servers = WS_SERVERS.lock().unwrap();
        if servers.contains_key(&room_id) {
            println!("🔁 Reusing existing WebSocketServer for room ID: {}", room_id);
        } else {
            println!("🆕 Creating new WebSocketServer for room ID: {}", room_id);
        }
        servers
            .entry(room_id.clone())
            .or_insert_with(|| Arc::new(Mutex::new(WebSocketServer::new())))
            .clone()
    };

    let connection_id = Uuid::new_v4().to_string();
    let user_type = user.map(|r| r.unwrap().r#type).unwrap().to_string();

    println!(
        "👤 Establishing WebSocket for user_type: {}, user_id: {}, connection_id: {}",
        user_type, user_id, connection_id
    );

    if user_type != UserType::Member.to_string() && user_type != UserType::Proctor.to_string() {
        println!("❌ Invalid user type: {}", user_type);
        return Ok(HttpResponse::BadRequest().body("Invalid user type"));
    }

    let actor = WebSocketActor::new(
        ws_server,
        connection_id,
        room_id,
        user_type,
        user_id,
        db_pool
    );

    ws::start(actor, &req, stream)
}
