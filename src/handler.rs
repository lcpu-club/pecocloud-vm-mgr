use crate::{
    pool::VmPool,
    model::*, operation::*,
};
/// handler for the routes
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use std::sync::Mutex;

#[get("/api/v1")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[get("/api/v1/vm")]
async fn get_root_vm_page_handler() -> impl Responder {
    HttpResponse::Ok().body("Index page of Vm management")
}

#[post("/api/v1/vm")]
async fn create_vm_handler(
    pool: web::Data<Mutex<VmPool>>,
    request: web::Json<VmCreateRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let res = create_vm_op(pool, &request.config).await;
    match res {
        Ok(vmid) => HttpResponse::Ok().json(VmCreateResponse {
            vmid,
            created_at: chrono::Local::now(),
        }),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/api/v1/vm/{vmid}")]
async fn get_vm_status_handler(
    pool: web::Data<Mutex<VmPool>>,
    request: web::Json<VmQueryStatusRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let res = get_vm_status_op(pool, request.vmid).await;
    match res {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[put("/api/v1/vm/{vmid}")]
async fn modify_metadata_handler(
    pool: web::Data<Mutex<VmPool>>,
    request: web::Json<VmModifyMetadataRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let res = modify_metadata_op(pool, request.vmid, &request.metadata).await;
    match res {
        Ok(_) => HttpResponse::Ok().json(VmModifyMetadataResponse {
            vmid: request.vmid,
            time: chrono::Local::now(),
        }),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[put("/api/v1/vm/{vmid}/power_state")]
async fn operate_vm_handler(
    pool: web::Data<Mutex<VmPool>>,
    request: web::Json<VmOperateRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let res = match request.operation {
        Operation::Start => start_vm_op(pool, request.vmid).await,
        Operation::Pause => pause_vm_op(pool, request.vmid).await,
        Operation::Resume => resume_vm_op(pool, request.vmid).await,
        Operation::Stop => stop_vm_op(pool, request.vmid).await,
    };
    match res {
        Ok(_) => HttpResponse::Ok().json(VmOperateResponse {
            vmid: request.vmid,
            time: chrono::Local::now(),
        }),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[delete("/api/v1/vm/delete")]
async fn delete_vm_handler(
    pool: web::Data<Mutex<VmPool>>,
    request: web::Json<VmDeleteRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let res = delete_vm_op(pool, request.vmid).await;
    match res {
        Ok(_) => HttpResponse::Ok().json(VmDeleteResponse {
            vmid: request.vmid,
            time: chrono::Local::now(),
        }),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/api/v1/vm/{vmid}/vm_mem_snapshot")]
async fn create_vm_mem_snapshot_handler(
    pool: web::Data<Mutex<VmPool>>,
    request: web::Json<VmCreateVMMSRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let res = create_vm_mem_snapshot_op(pool, request.vmid).await;
    match res {
        Ok(id) => HttpResponse::Ok().json(VmCreateVMMSResponse {
            vmid: request.vmid,
            vm_mem_snapshot_id: id,
        }),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[delete("/api/v1/vm/{vmid}/vm_mem_snapshot/{vm_mem_snapshot_id}")]
async fn delete_vm_mem_snapshot_handler(
    pool: web::Data<Mutex<VmPool>>,
    request: web::Json<VmDeleteVMMSRequest>,
) -> impl Responder {
    let request = request.into_inner();
    let res = delete_vm_mem_snapshot_op(pool, request.vmid, request.vm_mem_snapshot_id).await;
    match res {
        Ok(_) => HttpResponse::Ok().json(VmDeleteVMMSResponse {
            vmid: request.vmid,
            vm_mem_snapshot_id: request.vm_mem_snapshot_id,
        }),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
