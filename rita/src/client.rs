#![cfg_attr(feature = "system_alloc", feature(alloc_system, global_allocator, allocator_api))]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#[cfg(feature = "system_alloc")]
extern crate alloc_system;

#[cfg(feature = "system_alloc")]
use alloc_system::System;

#[cfg(feature = "system_alloc")]
#[global_allocator]
static A: System = System;

#[macro_use]
extern crate derive_error;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

extern crate actix;
extern crate actix_web;
extern crate bytes;
extern crate docopt;
extern crate env_logger;
extern crate eui48;
extern crate futures;
extern crate ip_network;
extern crate minihttpse;
extern crate rand;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate settings;
extern crate tokio;

use settings::Settings;
use docopt::Docopt;

use actix::*;
use actix::registry::SystemService;
use actix_web::*;

extern crate althea_kernel_interface;
extern crate althea_types;
extern crate babel_monitor;
extern crate num256;

mod debt_keeper;
mod payment_controller;
mod rita;

use rita::network_endpoints::{hello_response, make_payments};

const USAGE: &'static str = "
Usage: rita --config <settings> --default <default>
Options:
    --config   Name of config file
    --default   Name of default config file
";

lazy_static! {
    pub static ref SETTING: Settings = {
        let args = Docopt::new(USAGE)
        .and_then(|d| d.parse())
        .unwrap_or_else(|e| e.exit());

        let settings_file = args.get_str("<settings>");
        let defaults_file = args.get_str("<default>");

        let s = Settings::new(settings_file, defaults_file).unwrap();
        s.write(settings_file).unwrap();
        s
    };
}

fn main() {
    env_logger::init();
    trace!("Starting");
    trace!("Starting with Identity: {:?}", SETTING.get_identity());

    let system = actix::System::new(format!("main {}", SETTING.network.own_ip));

    assert!(debt_keeper::DebtKeeper::from_registry().connected());
    assert!(payment_controller::PaymentController::from_registry().connected());
    assert!(rita::tunnel_manager::TunnelManager::from_registry().connected());
    assert!(rita::http_client::HTTPClient::from_registry().connected());
    assert!(rita::traffic_watcher::TrafficWatcher::from_registry().connected());

    HttpServer::new(|| {
        Application::new()
            .resource("/make_payment", |r| r.h(make_payments))
            .resource("/hello", |r| r.h(hello_response))
    }).bind(format!("[::0]:{}", SETTING.network.rita_port))
        .unwrap()
        .start();

    let rita = rita::rita_loop::RitaLoop {};
    let _: Address<_> = rita.start();

    system.run();
}