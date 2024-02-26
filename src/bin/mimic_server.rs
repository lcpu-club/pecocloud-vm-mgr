use std::{default, env, sync::Mutex};

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use pecocloud_vm_mgr::{handler::*, pool::VmPool};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    log4rs::init_file("log4rs.yaml", default::Default::default()).unwrap();

    let pool = VmPool::new()
        .await
        .map_err(|e| {
            eprintln!("Fail to build a vm pool: {}", e.to_string());
            panic!();
        });

    let listen_address = match env::var("LISTENING_ADDR") {
        Ok(address) => address,
        Err(_) => {
            log::warn!("No bind address found in .env file, using 0.0.0.0");
            "0.0.0.0".to_owned()
        }
    };
    let port = match env::var("LISTENING_PORT") {
        Ok(port) => port.parse::<u16>().expect("Failed to parse port"),
        Err(_) => {
            log::warn!("No port found in .env file, using 58890");
            58890
        }
    };

    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(get_root_vm_page_handler)
            .service(create_vm_handler)
            .service(get_vm_status_handler)
            .service(modify_metadata_handler)
            .service(operate_vm_handler)
            .service(delete_vm_handler)
            .service(create_vm_mem_snapshot_handler)
            .service(delete_vm_mem_snapshot_handler)
            .app_data(web::Data::new(Mutex::new(pool.clone())))
    })
    .bind((listen_address, port))?
    .run()
    .await

}
