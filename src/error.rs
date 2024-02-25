use uuid::Uuid;

pub enum VmManageError {
    VmNotFound(Uuid),
    KernelNotFound(String),

    SerdeError,
    EtcdError,
    ReqwestError,
    IoError,

    DBConnection,
    DBDropTable,
    DBCreateTable,
    DBInsertion,
    DBDeleting,
    DBFetching,
    DBUpdating,

    MachineCreate,
    MachineDumpCore,
    MachineRebuild,
    MachineStart,
    MachinePause,
    MachineResume,
    MachineStop,
    MachineDelete,
    MachineQuery,

    VmMemSnapshotCreate,
    VmMemSnapshotDelete,

    EnvSocket,
    EnvLogDir,
    EnvMetricsDir,
    EnvAgentInit,
    EnvAgentRequest,
    EnvKernelList,
    EnvMemoryDir,
}

pub type VmManageResult<T> = Result<T, VmManageError>;

impl std::fmt::Display for VmManageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            VmManageError::VmNotFound(vmid) => format!("Vm {vmid} not found"),
            VmManageError::KernelNotFound(s) => format!("Kernel {s} not found"),
            VmManageError::EtcdError => format!("ETCD error"),
            VmManageError::ReqwestError => format!("Reqwest client error"),
            VmManageError::SerdeError => format!("Serde error"),
            VmManageError::IoError => format!("Io error"),
            VmManageError::DBConnection => format!("Connect database error"),
            VmManageError::DBCreateTable => format!("Create table error"),
            VmManageError::DBDropTable => format!("Drop table error"),
            VmManageError::DBInsertion => format!("Insert element error"),
            VmManageError::DBDeleting => format!("Delete element error"),
            VmManageError::DBFetching => format!("Fetch element error"),
            VmManageError::DBUpdating => format!("Updating element error"),
            VmManageError::MachineCreate => format!("Create machine error"),
            VmManageError::MachineDumpCore => format!("Dump machine error"),
            VmManageError::MachineRebuild => format!("Rebuild machine error"),
            VmManageError::MachineStart => format!("Start machine error"),
            VmManageError::MachinePause => format!("Pause machine error"),
            VmManageError::MachineResume => format!("Resume machine error"),
            VmManageError::MachineStop => format!("Stop machine error"),
            VmManageError::MachineDelete => format!("Delete machine error"),
            VmManageError::MachineQuery => format!("Query machine error"),
            VmManageError::VmMemSnapshotCreate => format!("Create vm/mem snapshot error"),
            VmManageError::VmMemSnapshotDelete => format!("Delete vm/mem snapshot error"),
            VmManageError::EnvSocket => format!("SOCKET_DIR must be set"),
            VmManageError::EnvLogDir => format!("LOGS_DIR must be set"),
            VmManageError::EnvMetricsDir => format!("METRICS_DIR must be set"),
            VmManageError::EnvAgentInit => format!("AGENT_INIT_TIMEOUT must be set"),
            VmManageError::EnvAgentRequest => format!("AGENT_AGENT_REQUEST_TIMEOUT_TIMEOUT must be set"),
            VmManageError::EnvKernelList => format!("KERNEL_LIST_FILE must be set"),
            VmManageError::EnvMemoryDir => format!("MEMORY_SNAPSHOT_DIR must be set"),
        };
        write!(f, "{}", s)
    }
}

impl From<serde_json::Error> for VmManageError {
    fn from(_e: serde_json::Error) -> Self {
        VmManageError::SerdeError
    }
}

impl From<etcd_client::Error> for VmManageError {
    fn from(_e: etcd_client::Error) -> Self {
        VmManageError::EtcdError
    }
}

impl From<reqwest::Error> for VmManageError {
    fn from(_e: reqwest::Error) -> Self {
        VmManageError::ReqwestError
    }
}

impl From<std::io::Error> for VmManageError {
    fn from(_e: std::io::Error) -> Self {
        VmManageError::IoError
    }
}