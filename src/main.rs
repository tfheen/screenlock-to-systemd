/*
   Copyright 2019 Tollef Fog Heen <tfheen@err.no>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#[macro_use]
extern crate log;

use dbus;

use dbus_tokio::connection;
use clap::{App};
use dbus::message::MatchRule;
use futures::{Future, StreamExt};

async fn activate_systemd_unit(conn: &dbus::nonblock::SyncConnection, unit_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let proxy = dbus::nonblock::Proxy::new("org.freedesktop.systemd1", "/org/freedesktop/systemd1", std::time::Duration::from_secs(2), conn);
    // TODO: Handle timeouts and errors here
    let (_x,): (dbus::Path,) = proxy.method_call("org.freedesktop.systemd1.Manager", "StartUnit", (unit_name, "replace",)).await.unwrap();
    Ok(())
}

async fn activate_lock_target(conn: &dbus::nonblock::SyncConnection) {
    let r = activate_systemd_unit(conn, "lock-activated.target").await;
    match r {
        Err(e) => error!("Failed to activate lock-activated.target: {}", e),
        Ok(_) => {}
    }
}

async fn activate_unlock_target(conn: &dbus::nonblock::SyncConnection) {
    let r = activate_systemd_unit(conn, "unlock-activated.target").await;
    match r {
        Err(e) => error!("Failed to activate unlock-activated.target: {}", e),
        Ok(_) => {}
    }
}

async fn screensaver_callback(conn: &dbus::nonblock::SyncConnection, screen_locked: bool) -> bool {
    info!("Screen lock event happened on the bus, screen locked: {}", screen_locked);
    if screen_locked {
        activate_lock_target(conn).await;
    } else {
        activate_unlock_target(conn).await;
    }
    true
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    App::new("systemd-lock-handler")
        .version("0.0")
        .author("Tollef Fog Heen <tfheen@err.no>")
        .about("Listens for various events and logs them")
        .get_matches();

    info!("Connecting to D-Bus");

    // Connect to the D-Bus session bus (this is blocking, unfortunately).
    let (resource, conn) = connection::new_session_sync()?;

    // The resource is a task that should be spawned onto a tokio compatible
    // reactor ASAP. If the resource ever finishes, you lost connection to D-Bus.
    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    info!("Connected, setting up matches");
    // To receive D-Bus signals we need to add match that defines which signals should be forwarded
    // to our application.
    let mut tokens = Vec::new();
    let mut signals = Vec::new();

    let mut streams =  futures::stream::FuturesUnordered::new();

    for intf in vec!["org.cinnamon.ScreenSaver", "org.gnome.ScreenSaver"] {
        let mr = MatchRule::new_signal(intf, "ActiveChanged");
        let mtc = conn.add_match(mr).await?;
        let token = mtc.token();
        let (incoming_signal, stream) = mtc.stream();
        let stream = stream.for_each(|(_, (screen_locked,)): (_, (bool,))| {
            let conn = conn.clone();
            info!("Hello from stream {} happened on the bus!", screen_locked);
            async move { screensaver_callback(&conn, screen_locked).await; }
        });
        signals.push(incoming_signal);
        streams.push(stream);
        tokens.push(token);
    }
    info!("bassfd");
    while let Some(_) = streams.next().await {info!("xx")}

    futures::future::pending::<()>().await;

    // Simultaneously run signal handling and method calling
    for token in tokens {
        conn.remove_match(token).await?;
    }
    unreachable!()
}
