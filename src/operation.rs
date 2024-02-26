use std::sync::Mutex;

use actix_web::web;
use uuid::Uuid;

use crate::{
    error::VmManageResult,
    model::*,
    pool::{self, VmPool},
};

pub async fn create_vm_op(
    pool: web::Data<Mutex<VmPool>>,
    create_config: &MachineCreateConfig,
) -> VmManageResult<Uuid> {
    let mut pool_mutex = pool.lock().unwrap();
    let vmid = Uuid::new_v4();
    let mut pool_guard = pool_mutex.lock(vmid).await?;
    let pool = pool_guard.pool();

    pool::create_vm(pool, vmid, create_config).await?;

    Ok(vmid)
}

pub async fn start_vm_op(pool: web::Data<Mutex<VmPool>>, vmid: Uuid) -> VmManageResult<()> {
    let mut pool_mutex = pool.lock().unwrap();
    let mut pool_guard = pool_mutex.lock(vmid).await?;
    let pool = pool_guard.pool();

    pool::start_vm(pool, vmid).await?;

    Ok(())
}

pub async fn pause_vm_op(pool: web::Data<Mutex<VmPool>>, vmid: Uuid) -> VmManageResult<()> {
    let mut pool_mutex = pool.lock().unwrap();
    let mut pool_guard = pool_mutex.lock(vmid).await?;
    let pool = pool_guard.pool();

    pool::pause_vm(pool, vmid).await?;

    Ok(())
}

pub async fn resume_vm_op(pool: web::Data<Mutex<VmPool>>, vmid: Uuid) -> VmManageResult<()> {
    let mut pool_mutex = pool.lock().unwrap();
    let mut pool_guard = pool_mutex.lock(vmid).await?;
    let pool = pool_guard.pool();

    pool::resume_vm(pool, vmid).await?;

    Ok(())
}

pub async fn stop_vm_op(pool: web::Data<Mutex<VmPool>>, vmid: Uuid) -> VmManageResult<()> {
    let mut pool_mutex = pool.lock().unwrap();
    let mut pool_guard = pool_mutex.lock(vmid).await?;
    let pool = pool_guard.pool();

    pool::stop_vm(pool, vmid).await?;

    Ok(())
}

pub async fn delete_vm_op(pool: web::Data<Mutex<VmPool>>, vmid: Uuid) -> VmManageResult<()> {
    let mut pool_mutex = pool.lock().unwrap();
    let mut pool_guard = pool_mutex.lock(vmid).await?;
    let pool = pool_guard.pool();

    pool::delete_vm(pool, vmid).await?;

    Ok(())
}

pub async fn modify_metadata_op(
    pool: web::Data<Mutex<VmPool>>,
    vmid: Uuid,
    metadata: &String,
) -> VmManageResult<()> {
    let mut pool_mutex = pool.lock().unwrap();
    let mut pool_guard = pool_mutex.lock(vmid).await?;
    let pool = pool_guard.pool();

    pool::modify_metadata(pool, vmid, metadata).await?;

    Ok(())
}

pub async fn get_vm_status_op(
    pool: web::Data<Mutex<VmPool>>,
    vmid: Uuid,
) -> VmManageResult<VmViewInfo> {
    let mut pool_mutex = pool.lock().unwrap();
    let mut pool_guard = pool_mutex.lock(vmid).await?;
    let pool = pool_guard.pool();

    let infos = pool::get_vm_status(pool, vmid).await?;

    Ok(infos)
}

pub async fn create_vm_mem_snapshot_op(pool: web::Data<Mutex<VmPool>>, vmid: Uuid) -> VmManageResult<Uuid> {
    let mut pool_mutex = pool.lock().unwrap();
    let mut pool_guard = pool_mutex.lock(vmid).await?;
    let pool = pool_guard.pool();

    let vm_mem_snapshot_id = pool::create_vm_mem_snapshot(pool, vmid).await?;

    Ok(vm_mem_snapshot_id)
}

pub async fn delete_vm_mem_snapshot_op(pool: web::Data<Mutex<VmPool>>, vmid: Uuid, vm_mem_snapshot_id: Uuid) -> VmManageResult<()> {
    let mut pool_mutex = pool.lock().unwrap();
    let mut pool_guard = pool_mutex.lock(vmid).await?;
    let pool = pool_guard.pool();

    pool::delete_vm_mem_snapshot(pool, vmid, vm_mem_snapshot_id).await?;
    Ok(())
}