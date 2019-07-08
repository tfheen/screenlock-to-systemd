#[macro_use]
extern crate log;

use dbus;

use std::rc::Rc;
use tokio::reactor::Handle;
use tokio::runtime::current_thread::Runtime;
use futures::{Stream};
use dbus_tokio::AConnection;
use clap::{App};

fn activate_systemd_unit(conn: &Rc<dbus::Connection>, unit_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let m = dbus::Message::new_method_call("org.freedesktop.systemd1", "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager", "StartUnit")?
        .append2(unit_name, "replace");
    conn.send_with_reply_and_block(m, 2000)?;
    Ok(())
}

fn activate_lock_target(conn: &Rc<dbus::Connection>) {
    let r = activate_systemd_unit(conn, "lock-activated.target");
    match r {
        Err(e) => error!("Failed to activate lock-activated.target: {}", e),
        Ok(_) => {},
    }
}

fn activate_unlock_target(conn: &Rc<dbus::Connection>) {
    let r = activate_systemd_unit(conn, "unlock-activated.target");
    match r {
        Err(e) => error!("Failed to activate unlock-activated.target: {}", e),
        Ok(_) => {},
    }
}

fn main() {
    env_logger::init();

    App::new("systemd-lock-handler")
        .version("0.0")
        .author("Tollef Fog Heen <tfheen@err.no>")
        .about("Listens for various events and logs them")
        .get_matches();

    info!("Connecting to D-Bus");

    // Let's start by starting up a connection to the session bus. We do not register a name
    // because we do not intend to expose any objects on the bus.
    let c = Rc::new(dbus::Connection::get_private(dbus::BusType::Session).unwrap());

    // To receive D-Bus signals we need to add match that defines which signals should be forwarded
    // to our application.
    c.add_match("type=signal,sender=org.cinnamon.ScreenSaver,member=ActiveChanged").unwrap();

    // Create Tokio event loop along with asynchronous connection object
    let mut rt = Runtime::new().unwrap();
    let aconn = AConnection::new(c.clone(), Handle::default(), &mut rt).unwrap();

    // Create stream of all incoming D-Bus messages. On top of the messages stream create future,
    // running forever, handling all incoming messages
    let messages = aconn.messages().unwrap();
    let signals = messages.for_each(|m| {
        let headers = m.headers();
        let member = headers.3.unwrap();
        if member == "ActiveChanged" {
            let screen_locked : bool = m.get1().unwrap();
            info!("Screen lock event happened on the bus, screen locked: {}", screen_locked);
            if screen_locked {
                activate_lock_target(&c);
            } else {
                activate_unlock_target(&c);
            }
        } else {
            debug!("Ignored message: {:?}", m)
        }
        Ok(())
    });

    // Simultaneously run signal handling and method calling
    rt.block_on(signals).unwrap();
}
