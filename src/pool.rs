use std::{env, path::PathBuf};

use rustcracker::{
    components::machine::{Config, Machine, MachineCore},
    model::{drive::Drive, logger::LogLevel, machine_configuration::MachineConfiguration},
};
use sqlx::postgres;
use uuid::Uuid;

use crate::{
    error::{VmManageError, VmManageResult},
    kernel_mgr::get_kernel_image_path,
    model::*,
    sql::*,
    storage_mgr::*,
};

#[derive(Clone)]
pub struct VmPool {
    pub pool_id: Uuid,
    pub conn: postgres::PgPool,
    pub etcd_client: etcd_client::Client,
    pub storage_mgr_addr: String,
    pub storage_client: reqwest::Client,
    pub network_mgr_addr: String,
    pub network_client: reqwest::Client,
    pub socket_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub metrics_dir: PathBuf,
    pub memory_snapshot_dir: PathBuf,
}

pub struct VmPoolGuard<'a> {
    pub pool: &'a mut VmPool,
    pub lock: Option<String>,
}

impl<'a> Drop for VmPoolGuard<'a> {
    fn drop(&mut self) {
        if let Some(lock) = self.lock.take() {
            let mut pool = self.pool.clone();
            tokio::spawn(async move {
                if let Err(e) = release_vm_lock(&mut pool, &lock).await {
                    log::error!("Failed to release lock {}: {}", lock, e)
                }
            });
        }
    }
}

impl<'a> VmPoolGuard<'a> {
    pub fn new(pool: &mut VmPool, lock: String) -> VmPoolGuard {
        VmPoolGuard {
            pool,
            lock: Some(lock),
        }
    }

    pub fn pool(&mut self) -> &mut VmPool {
        self.pool
    }
}

impl VmPool {
    pub async fn new() -> VmManageResult<VmPool> {
        let socket_dir =
            PathBuf::from(env::var("SOCKETS_DIR").map_err(|_| VmManageError::EnvSocket)?);
        let logs_dir = PathBuf::from(env::var("LOGS_DIR").map_err(|_| VmManageError::EnvLogDir)?);
        let metrics_dir =
            PathBuf::from(env::var("METRICS_DIR").map_err(|_| VmManageError::EnvMetricsDir)?);
        let memory_snapshot_dir = PathBuf::from(
            env::var("MEMORY_SNAPSHOT_DIR").map_err(|_| VmManageError::EnvMemoryDir)?,
        );

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let database_user = env::var("DATABASE_USER").expect("DATABASE_USER must be set");
        let database_password =
            env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD must be set");
        let database_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");

        let etcd_url = env::var("ETCD_URL").expect("ETCD_URL must be set");
        let etcd_user = env::var("ETCD_USER").expect("ETCD_USER must be set");
        let etcd_password = env::var("ETCD_PASSWORD").expect("ETCD_PASSWORD must be set");
        let _etcd_prefix = env::var("ETCD_PREFIX").expect("ETCD_PREFIX must be set");

        let database_url = format!(
            "postgres://{}:{}@{}/{}",
            database_user, database_password, database_url, database_name
        );
        log::debug!("Database URL: {}", database_url);

        let conn = postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await
            .map_err(|_| VmManageError::DBConnection)?;

        let etcd_config = etcd_client::ConnectOptions::new().with_user(etcd_user, etcd_password);
        let etcd_client = etcd_client::Client::connect([etcd_url], Some(etcd_config)).await?;

        let storage_mgr_addr = env::var("STORAGE_MGR_ADDR").expect("STORAGE_MGR_ADDR must be set");
        let storage_client = reqwest::Client::new();
        let network_mgr_addr = env::var("NETWORK_MGR_ADDR").expect("NETWORK_MGR_ADDR must be set");
        let network_client = reqwest::Client::new();
        let pool_id = Uuid::new_v4();

        let pool = VmPool {
            pool_id,
            conn,
            etcd_client,
            storage_mgr_addr,
            storage_client,
            network_mgr_addr,
            network_client,
            socket_dir,
            logs_dir,
            metrics_dir,
            memory_snapshot_dir,
        };

        init_pool(pool).await
    }

    pub async fn lock(&mut self, vmid: Uuid) -> VmManageResult<VmPoolGuard> {
        let lock = get_vm_lock(self, vmid, None).await?;
        Ok(VmPoolGuard {
            pool: self,
            lock: Some(lock),
        })
    }
}

impl VmPool {
    #[inline]
    fn machine_core_storage_table(&self) -> String {
        format!(
            "{}_{}",
            std::env::var(MACHINE_CORE_TABLE_NAME)
                .unwrap_or(DEFAULT_MACHINE_CORE_TABLE.to_string()),
            self.pool_id
        )
    }

    #[inline]
    fn config_storage_table(&self) -> String {
        format!(
            "{}_{}",
            std::env::var(VM_CONFIG_TABLE_NAME).unwrap_or(DEFAULT_VM_CONFIG_TABLE.to_string()),
            self.pool_id
        )
    }

    #[inline]
    fn vm_mem_snapshot_storage_table(&self) -> String {
        format!(
            "{}_{}",
            std::env::var(VM_MEM_SNAPSHOT_TABLE_NAME)
                .unwrap_or(DEFAULT_VM_MEM_SNAPSHOT_TABLE.to_string()),
            self.pool_id
        )
    }

    #[inline]
    fn volume_storage_table(&self) -> String {
        format!(
            "{}_{}",
            std::env::var(VOLUME_TABLE_NAME).unwrap_or(DEFAULT_VOLUME_TABLE.to_string()),
            self.pool_id
        )
    }

    #[inline]
    fn socket_path(&self, vmid: Uuid) -> PathBuf {
        self.socket_dir
            .join(self.pool_id.to_string())
            .join(format!("{}.socket", vmid))
    }

    #[inline]
    fn log_fifo(&self, vmid: Uuid) -> PathBuf {
        self.logs_dir
            .join(self.pool_id.to_string())
            .join(format!("{}.log", vmid))
    }

    #[inline]
    fn metrics_fifo(&self, vmid: Uuid) -> PathBuf {
        self.metrics_dir
            .join(self.pool_id.to_string())
            .join(format!("{}.metrics", vmid))
    }

    #[inline]
    fn mem_snapshot_path(&self, vmid: Uuid, vm_mem_snapshot_id: Uuid) -> PathBuf {
        self.memory_snapshot_dir
            .join(self.pool_id.to_string())
            .join(vmid.to_string())
            .join(format!("{}.mem", vm_mem_snapshot_id))
    }

    #[inline]
    fn vm_snapshot_path(&self, vmid: Uuid, vm_mem_snapshot_id: Uuid) -> PathBuf {
        self.memory_snapshot_dir
            .join(self.pool_id.to_string())
            .join(vmid.to_string())
            .join(format!("{}.vm", vm_mem_snapshot_id))
    }
}

impl VmPool {
    async fn get_core_db(&self, vmid: Uuid) -> VmManageResult<MachineCore> {
        log::trace!("Getting core of {} from database", vmid);
        let core = sqlx::query_as::<_, PgMachineCoreElement>(GET_MACHINE_CORE_BY_VMID)
            .bind(self.machine_core_storage_table())
            .bind(vmid)
            .fetch_one(&self.conn)
            .await
            .map_err(|_| VmManageError::DBFetching)?;
        let core = core.core.0;
        Ok(core)
    }

    async fn add_core_db(
        &self,
        vmid: Uuid,
        core: &MachineCore,
        status: MachineState,
    ) -> VmManageResult<()> {
        log::trace!("Adding core of {} to database", vmid);
        let machine_core_storage_table = self.machine_core_storage_table();
        sqlx::query(INSERT_MACHINE_CORE_BY_VMID)
            .bind(machine_core_storage_table)
            .bind(vmid)
            .bind(sqlx::types::Json(core.to_owned()))
            .bind(sqlx::types::Json(status))
            .execute(&self.conn)
            .await
            .map_err(|_| VmManageError::DBInsertion)?;

        Ok(())
    }

    async fn delete_core_db(&self, vmid: Uuid) -> VmManageResult<()> {
        log::trace!("Deleting core of {} from database", vmid);
        let machine_core_storage_table = self.machine_core_storage_table();
        sqlx::query(DELETE_MACHINE_CORE_BY_VMID)
            .bind(machine_core_storage_table)
            .bind(vmid)
            .execute(&self.conn)
            .await
            .map_err(|_| VmManageError::DBDeleting)?;
        Ok(())
    }

    async fn update_core_db(
        &self,
        vmid: Uuid,
        core: &MachineCore,
        status: MachineState,
    ) -> VmManageResult<()> {
        log::trace!("Updating status of {}", vmid);
        sqlx::query(UPDATE_MACHINE_CORE_BY_VMID)
            .bind(self.machine_core_storage_table())
            .bind(sqlx::types::Json(core))
            .bind(status)
            .bind(vmid)
            .execute(&self.conn)
            .await
            .map_err(|_| VmManageError::DBUpdating)?;
        Ok(())
    }

    async fn add_create_config_db(
        &self,
        vmid: Uuid,
        config: &MachineCreateConfig,
    ) -> VmManageResult<()> {
        log::trace!("Adding create-config of {} to database", vmid);
        let config_storage_table = self.config_storage_table();
        sqlx::query(INSERT_VMVIEWCONFIGS_BY_VMID)
            .bind(config_storage_table)
            .bind(vmid)
            .bind(sqlx::types::Json(config))
            .execute(&self.conn)
            .await
            .map_err(|_| VmManageError::DBInsertion)?;

        Ok(())
    }

    async fn delete_create_config_db(&self, vmid: Uuid) -> VmManageResult<()> {
        log::trace!("Deleting create-config of {} from database", vmid);
        let config_storage_table = self.config_storage_table();
        sqlx::query(DELETE_VMVIEWCONFIGS_BY_VMID)
            .bind(config_storage_table)
            .bind(vmid)
            .execute(&self.conn)
            .await
            .map_err(|_| VmManageError::DBDeleting)?;

        Ok(())
    }

    async fn add_volume_db(&self, vmid: Uuid, volume: Uuid) -> VmManageResult<()> {
        log::trace!("Adding volume {} of vm {} to database", volume, vmid);
        let volume_storage_table = self.volume_storage_table();
        sqlx::query(INSERT_VOLUME_BY_ID)
            .bind(volume_storage_table)
            .bind(vmid)
            .bind(volume)
            .execute(&self.conn)
            .await
            .map_err(|_| VmManageError::DBInsertion)?;

        Ok(())
    }

    async fn delete_volume_db(&self, vmid: Uuid, volume: Uuid) -> VmManageResult<()> {
        log::trace!("Deleting volume {} of vm {} from database", volume, vmid);
        let volume_storage_table = self.volume_storage_table();
        sqlx::query(DELETE_VOLUME_BY_ID)
            .bind(volume_storage_table)
            .bind(vmid)
            .bind(volume)
            .execute(&self.conn)
            .await
            .map_err(|_| VmManageError::DBDeleting)?;

        Ok(())
    }

    async fn get_volume_id(&self, vmid: Uuid) -> VmManageResult<Vec<Uuid>> {
        log::trace!("Getting volume id of vm {} from database", vmid);
        let volume_storage_table = self.volume_storage_table();
        let elements: Vec<Uuid> = sqlx::query_as::<_, PgVolumeElement>(GET_VOLUME_ALL)
            .bind(volume_storage_table)
            .bind(vmid)
            .fetch_all(&self.conn)
            .await
            .map_err(|_| VmManageError::DBFetching)?
            .into_iter()
            .map(|x| x.volume_id)
            .collect();

        Ok(elements)
    }

    async fn add_vm_mem_snapshot_db(
        &self,
        vmid: Uuid,
        vm_mem_snapshot_id: Uuid,
        mem_file_path: &PathBuf,
        snapshot_path: &PathBuf,
    ) -> VmManageResult<()> {
        log::trace!(
            "Adding vm/mem snapshot {} of vm {} to database",
            vm_mem_snapshot_id,
            vmid
        );
        let vm_mem_snapshot_storage_table = self.vm_mem_snapshot_storage_table();
        sqlx::query(INSERT_VM_MEM_SNAPSHOT_BY_ID)
            .bind(vm_mem_snapshot_storage_table)
            .bind(vmid)
            .bind(vm_mem_snapshot_id)
            .bind(mem_file_path.to_str())
            .bind(snapshot_path.to_str())
            .execute(&self.conn)
            .await
            .map_err(|_| VmManageError::DBInsertion)?;

        Ok(())
    }

    async fn delete_vm_mem_snapshot_db(
        &self,
        vmid: Uuid,
        vm_mem_snapshot_id: Uuid,
    ) -> VmManageResult<()> {
        log::trace!(
            "Deleting  vm/mem snapshot {} of vm {} from database",
            vm_mem_snapshot_id,
            vmid
        );
        let vm_mem_snapshot_storage_table = self.vm_mem_snapshot_storage_table();
        sqlx::query(DELETE_VM_MEM_SNAPSHOT_BY_ID)
            .bind(vm_mem_snapshot_storage_table)
            .bind(vmid)
            .bind(vm_mem_snapshot_id)
            .execute(&self.conn)
            .await
            .map_err(|_| VmManageError::DBDeleting)?;

        Ok(())
    }
}

async fn init_pool(pool: VmPool) -> VmManageResult<VmPool> {
    /* Create tables for storing */
    /* machine core */
    log::trace!("Initializing postgres");
    sqlx::query(DROP_MAHCINE_CORE_TABLE_SQL)
        .bind(pool.machine_core_storage_table())
        .execute(&pool.conn)
        .await
        .map_err(|_| VmManageError::DBDropTable)?;
    sqlx::query(CREATE_MACHINE_CORE_TABLE_SQL)
        .bind(pool.machine_core_storage_table())
        .execute(&pool.conn)
        .await
        .map_err(|_| VmManageError::DBCreateTable)?;
    /* vm, mem snapshots */
    sqlx::query(DROP_VM_MEM_SNAPSHOT_TABLE_SQL)
        .bind(pool.vm_mem_snapshot_storage_table())
        .execute(&pool.conn)
        .await
        .map_err(|_| VmManageError::DBDropTable)?;
    sqlx::query(CREATE_VM_MEM_SNAPSHOT_TABLE_SQL)
        .bind(pool.vm_mem_snapshot_storage_table())
        .execute(&pool.conn)
        .await
        .map_err(|_| VmManageError::DBCreateTable)?;
    /* vm configs */
    sqlx::query(DROP_VMVIEWCONFIGS_TABLE_SQL)
        .bind(pool.config_storage_table())
        .execute(&pool.conn)
        .await
        .map_err(|_| VmManageError::DBDropTable)?;
    sqlx::query(CREATE_VMVIEWCONFIGS_TABLE_SQL)
        .bind(pool.config_storage_table())
        .execute(&pool.conn)
        .await
        .map_err(|_| VmManageError::DBCreateTable)?;
    /* volumes */
    sqlx::query(DROP_VOLUME_TABLE_SQL)
        .bind(pool.volume_storage_table())
        .execute(&pool.conn)
        .await
        .map_err(|_| VmManageError::DBDropTable)?;
    sqlx::query(CREATE_VOLUME_TABLE_SQL)
        .bind(pool.volume_storage_table())
        .execute(&pool.conn)
        .await
        .map_err(|_| VmManageError::DBCreateTable)?;

    /* Check storage mgr */
    log::trace!("Checking stroage mgr");
    let url = format!("{}{}", pool.storage_mgr_addr, "/api/v1");
    let status = pool.storage_client.get(url).send().await?.status();
    match status.is_success() {
        true => log::trace!("Check storage mgr success"),
        false => {
            log::error!("Check storage mgr fail");
            return Err(VmManageError::ReqwestError);
        }
    }

    /* Ping to network mgr */
    log::trace!("Checking network mgr");
    let url = format!("{}{}", pool.network_mgr_addr, "/api/v1");
    let status = pool.network_client.get(url).send().await?.status();
    match status.is_success() {
        true => log::trace!("Check network mgr success"),
        false => {
            log::error!("Check network mgr fail");
            return Err(VmManageError::ReqwestError);
        }
    }

    Ok(pool)
}

async fn get_vm_lock(pool: &mut VmPool, vmid: Uuid, lease: Option<i64>) -> VmManageResult<String> {
    let lock_name = format!("/lock/vm/{}", vmid);

    log::trace!("Getting lock for vm {}", vmid);

    let resp = pool
        .etcd_client
        .lease_grant(lease.unwrap_or(120), None)
        .await?;
    let lease_id = resp.id();
    let lock_options = etcd_client::LockOptions::new().with_lease(lease_id);

    let resp = pool.etcd_client.lock(lock_name, Some(lock_options)).await?;
    let key = resp.key();
    let key_str = std::str::from_utf8(key).unwrap().to_owned();

    log::trace!("Got lock {} for volume {}", key_str, vmid);

    Ok(key_str)
}

async fn release_vm_lock(pool: &mut VmPool, lock: &str) -> VmManageResult<()> {
    log::trace!("Releasing lock {}", lock);

    pool.etcd_client.unlock(lock).await?;
    Ok(())
}

async fn get_vm(pool: &mut VmPool, vmid: Uuid) -> VmManageResult<Machine> {
    log::trace!("Getting vm {}", vmid);

    /* Fetch metadata from database */
    let core = pool.get_core_db(vmid).await?;

    /* Rebuild machine agent */
    let machine = Machine::rebuild(core).map_err(|_| VmManageError::MachineRebuild)?;
    Ok(machine)
}

pub async fn create_vm(
    pool: &mut VmPool,
    vmid: Uuid,
    create_config: &MachineCreateConfig,
) -> VmManageResult<Uuid> {
    log::trace!("Creating vm {}", vmid);
    /* Read critical parameters */
    let socket_path = pool.socket_path(vmid);
    let log_fifo = pool.log_fifo(vmid);
    let metrics_fifo = pool.metrics_fifo(vmid);
    let agent_init_timeout = env::var("AGENT_INIT_TIMEOUT")
        .map_err(|_| VmManageError::EnvAgentInit)?
        .parse::<f64>()
        .map_err(|_| VmManageError::EnvAgentInit)?;
    let agent_request_timeout = env::var("AGENT_INIT_TIMEOUT")
        .map_err(|_| VmManageError::EnvAgentRequest)?
        .parse::<f64>()
        .map_err(|_| VmManageError::EnvAgentRequest)?;

    /* Machine configuration for the microVM */
    let machine_cfg = MachineConfiguration {
        cpu_template: None,
        ht_enabled: create_config.enable_hyperthreading,
        mem_size_mib: create_config.memory_size_in_mib as isize,
        track_dirty_pages: None,
        vcpu_count: create_config.vcpu_count as isize,
    };

    /* Desired kernel image */
    let kernel_image_path =
        get_kernel_image_path(&create_config.kernel_name, &create_config.kernel_version)?;

    /* Request for a volume from storage manager */
    let volume_id = create_volume(pool, create_config.volume_size_in_mib, None).await?;
    let volume_path = attach_volume(pool, volume_id).await?;

    /* Config the root device using the volume */
    let root_device = Drive {
        drive_id: "rootfs".to_string(),
        partuuid: Some(volume_id.to_string()),
        is_root_device: true,
        cache_type: None,
        is_read_only: false,
        path_on_host: PathBuf::from(volume_path),
        rate_limiter: None,
        io_engine: None,
        socket: None,
    };

    /* Build the config */
    let config = Config {
        socket_path: Some(socket_path),
        log_fifo: Some(log_fifo),
        log_path: None,
        log_level: Some(LogLevel::Info), // Set log level to Info
        log_clear: Some(false),          // Keep log fifo
        metrics_fifo: Some(metrics_fifo),
        metrics_path: None,
        metrics_clear: Some(false), // Keep metrics fifo
        kernel_image_path: Some(kernel_image_path),
        initrd_path: None,
        kernel_args: None,
        drives: Some(vec![root_device]), // Root device
        network_interfaces: None,        // TODO: network interface
        vsock_devices: None,
        machine_cfg: Some(machine_cfg),
        disable_validation: true, // Enable validation
        enable_jailer: false,     // Disable jailer
        jailer_cfg: None,
        vmid: None,
        net_ns: None,
        network_clear: Some(true),
        forward_signals: None,
        seccomp_level: None,
        mmds_address: None,
        balloon: None,
        init_metadata: create_config.initial_metadata.to_owned(), // Initial metadata
        stderr: None,
        stdin: None,
        stdout: None,
        agent_init_timeout: Some(agent_init_timeout),
        agent_request_timeout: Some(agent_request_timeout),
    }; // Config

    /* Assemble database metadata */
    /* Create the machine */
    let machine = Machine::new(config.to_owned()).map_err(|_| VmManageError::MachineCreate)?;

    /* Dump to machine core */
    let core = machine
        .dump_into_core()
        .map_err(|_| VmManageError::MachineDumpCore)?;

    /* Add the creating config to database */
    pool.add_create_config_db(vmid, create_config).await?;

    /* Add core to database */
    pool.add_core_db(vmid, &core, CREATED).await?;

    /* Add volume to database */
    pool.add_volume_db(vmid, volume_id).await?;

    log::trace!("Created vm {}", vmid);
    Ok(vmid)
}

pub async fn start_vm(pool: &mut VmPool, vmid: Uuid) -> VmManageResult<()> {
    log::trace!("Starting vm");
    let mut machine = get_vm(pool, vmid).await?;
    machine
        .start()
        .await
        .map_err(|_| VmManageError::MachineStart)?;
    let core = machine
        .dump_into_core()
        .map_err(|_| VmManageError::MachineDumpCore)?;
    pool.update_core_db(vmid, &core, RUNNING).await?;
    Ok(())
}

pub async fn pause_vm(pool: &mut VmPool, vmid: Uuid) -> VmManageResult<()> {
    log::trace!("Pausing vm {}", vmid);
    let machine = get_vm(pool, vmid).await?;
    machine
        .pause()
        .await
        .map_err(|_| VmManageError::MachinePause)?;
    let core = machine
        .dump_into_core()
        .map_err(|_| VmManageError::MachineDumpCore)?;
    pool.update_core_db(vmid, &core, PAUSED).await?;
    Ok(())
}

pub async fn resume_vm(pool: &mut VmPool, vmid: Uuid) -> VmManageResult<()> {
    log::trace!("Resuming vm {}", vmid);
    let machine = get_vm(pool, vmid).await?;
    machine
        .resume()
        .await
        .map_err(|_| VmManageError::MachineResume)?;
    let core = machine
        .dump_into_core()
        .map_err(|_| VmManageError::MachineDumpCore)?;
    pool.update_core_db(vmid, &core, RUNNING).await?;
    Ok(())
}

pub async fn stop_vm(pool: &mut VmPool, vmid: Uuid) -> VmManageResult<()> {
    log::trace!("Stopping vm {}", vmid);
    let machine = get_vm(pool, vmid).await?;
    machine
        .shutdown()
        .await
        .map_err(|_| VmManageError::MachineStop)?;
    let core = machine
        .dump_into_core()
        .map_err(|_| VmManageError::MachineDumpCore)?;
    pool.update_core_db(vmid, &core, STOPPED).await?;
    Ok(())
}

pub async fn delete_vm(pool: &mut VmPool, vmid: Uuid) -> VmManageResult<()> {
    log::trace!("Deleting vm {}", vmid);
    let mut machine = get_vm(pool, vmid).await?;
    let _ = machine.shutdown().await;
    machine
        .stop_vmm()
        .await
        .map_err(|_| VmManageError::MachineStop)?;
    let core = machine
        .dump_into_core()
        .map_err(|_| VmManageError::MachineDumpCore)?;
    pool.update_core_db(vmid, &core, DELETED).await?;

    /* Remove settings from db */
    /* Delete the creating config to database */
    pool.delete_create_config_db(vmid).await?;

    /* Delete core from database */
    pool.delete_core_db(vmid).await?;

    /* Get every volume_id from database */
    let volume_ids = pool.get_volume_id(vmid).await?;

    /* Delete volume from database */
    for volume_id in volume_ids {
        pool.delete_volume_db(vmid, volume_id).await?;

        /* Detach */
        detach_volume(pool, volume_id).await?;

        /* Delete */
        delete_volume(pool, volume_id).await?;
    }

    Ok(())
}

pub async fn modify_metadata(
    pool: &mut VmPool,
    vmid: Uuid,
    metadata: &String,
) -> VmManageResult<()> {
    log::trace!("Modifying metadata of {}", vmid);
    let machine = get_vm(pool, vmid).await?;
    machine
        .update_metadata(metadata)
        .await
        .map_err(|_| VmManageError::MachinePause)?;
    Ok(())
}

pub async fn get_vm_status(pool: &mut VmPool, vmid: Uuid) -> VmManageResult<VmViewInfo> {
    log::trace!("Getting vm status of {}", vmid);
    let mut machine = get_vm(pool, vmid).await?;
    let full_config = machine
        .get_export_vm_config()
        .await
        .map_err(|_| VmManageError::MachineQuery)?;
    let vm_info = machine
        .describe_instance_info()
        .await
        .map_err(|_| VmManageError::MachineQuery)?;
    let boot_config = machine.get_config();
    let vm_status = VmViewInfo {
        vmid,
        vm_info,
        full_config,
        boot_config,
    };

    Ok(vm_status)
}

pub async fn create_vm_mem_snapshot(pool: &mut VmPool, vmid: Uuid) -> VmManageResult<Uuid> {
    log::trace!("Creating vm/mem snapshot for {}", vmid);
    let vm_mem_snapshot_id = Uuid::new_v4();
    let mem_snapshot_path = pool.mem_snapshot_path(vmid, vm_mem_snapshot_id);
    let vm_snapshot_path = pool.vm_snapshot_path(vmid, vm_mem_snapshot_id);

    let machine = get_vm(pool, vmid).await?;
    machine
        .create_snapshot(&mem_snapshot_path, &vm_snapshot_path)
        .await
        .map_err(|_| VmManageError::VmMemSnapshotCreate)?;
    pool.add_vm_mem_snapshot_db(
        vmid,
        vm_mem_snapshot_id,
        &mem_snapshot_path,
        &vm_snapshot_path,
    )
    .await?;

    Ok(vm_mem_snapshot_id)
}

pub async fn delete_vm_mem_snapshot(
    pool: &mut VmPool,
    vmid: Uuid,
    vm_mem_snapshot_id: Uuid,
) -> VmManageResult<()> {
    log::trace!(
        "Deleting vm/mem snapshot {} of {}",
        vm_mem_snapshot_id,
        vmid
    );
    let mem_snapshot_path = pool.mem_snapshot_path(vmid, vm_mem_snapshot_id);
    let vm_snapshot_path = pool.vm_snapshot_path(vmid, vm_mem_snapshot_id);

    std::fs::remove_file(mem_snapshot_path).map_err(|_| VmManageError::IoError)?;
    std::fs::remove_file(vm_snapshot_path).map_err(|_| VmManageError::IoError)?;

    pool.delete_vm_mem_snapshot_db(vmid, vm_mem_snapshot_id)
        .await?;

    Ok(())
}
