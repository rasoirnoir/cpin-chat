#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket::response::stream::Event;
use rocket::response::stream::EventStream;
use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::channel;
use rocket::tokio::sync::broadcast::error::RecvError;
use rocket::tokio::sync::broadcast::Sender;
use rocket::Shutdown;
use rocket::State;

#[post("/message", data = "<form>")]
fn post(form: Form<Message>, queue: &State<Sender<Message>>) {
    let _res = queue.send(form.into_inner());
}

#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();

    EventStream! {
        loop{
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };
            yield Event::json(&msg);
        }
    }
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    #[field(validate = len(0..30))]
    pub room: String,
    #[field(validate = len(0..20))]
    pub username: String,
    pub message: String,
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(channel::<Message>(1024).0)
        .mount("/", routes![post, events])
}
