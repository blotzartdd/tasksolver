use chashmap::CHashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use super::models::responses::GetStatusResponse;
use super::routes::routes_handler;
use crate::server::models::responses::TaskStatusEnum;
use crate::worker_pool::worker_pool::WorkerPool;

use tokio::task::{self, JoinHandle};
use uuid::Uuid;
use warp;

use chrono::prelude::*;

/// Task status hashmap for all tasks on server
#[derive(Clone)]
pub struct TaskStatus {
    task_status_chashmap: Arc<CHashMap<String, GetStatusResponse>>,
}

impl TaskStatus {
    /// Create task status struct
    pub fn new() -> TaskStatus {
        TaskStatus {
            task_status_chashmap: Arc::new(CHashMap::new()),
        }
    }

    pub fn get_status_by_id(&self, id: &str) -> GetStatusResponse {
        if let Some(status) = self.task_status_chashmap.get(id) {
            return status.clone();
        }

        let error_status = GetStatusResponse::new_error_status();
        error_status
    }

    pub fn add_new_task(&self) -> String {
        let status = GetStatusResponse::new_utc_status();
        let id = Uuid::new_v4().to_string();
        self.task_status_chashmap.insert(id.clone(), status);

        id
    }

    pub fn start_running_task(&mut self, id: &str) {
        let mut status = self.task_status_chashmap.get_mut(id).unwrap();
        status.status = TaskStatusEnum::RUNNING;
        status.meta.started_at = Some(Utc::now().to_string());
    }

    pub fn finish_running_task(
        &mut self,
        id: &str,
        stdout: String,
        stderr: Option<String>,
        execution_result: TaskStatusEnum,
    ) {
        let mut status = self.task_status_chashmap.get_mut(id).unwrap();
        status.result.stdout = stdout;
        status.result.stderr = stderr;
        status.status = execution_result;
        status.meta.finished_at = Some(Utc::now().to_string());
    }
}

/// Struct of server info that contains
/// thread pool with workers, server queue of tasks
/// and status of all tasks.
#[derive(Clone)]
pub struct ServerInfo {
    pub worker_pool: Arc<WorkerPool>,
    pub task_status: TaskStatus,
}

impl ServerInfo {
    /// Creates new server info struct
    pub fn new(worker_pool: Arc<WorkerPool>, task_status: TaskStatus) -> ServerInfo {
        ServerInfo {
            worker_pool,
            task_status,
        }
    }
}

/// TaskSolver server struct
pub struct TaskSolverServer {
    socket: SocketAddr,
    server_info: ServerInfo,
}

impl TaskSolverServer {
    /// Creates new task solver server with given workers count, ip and port
    pub fn new(workers_count: usize, ip: String, port: u16) -> TaskSolverServer { 
        let socket = SocketAddr::new(ip.parse().unwrap(), port);

        let (task_sender, task_receiver) = async_channel::unbounded();
        let worker_pool = Arc::new(WorkerPool::new(workers_count, task_sender, task_receiver));

        let task_status = TaskStatus::new();
        let server_info = ServerInfo::new(worker_pool.clone(), task_status);

        TaskSolverServer {
            socket,
            server_info,
        }
    }
    /// Runs server on given ip and port. Creates worker pool with given
    /// amount of workers. Creates tokio threads to manage the server and task queue in parallel.
    pub async fn start_tasksolver_server(self) -> JoinHandle<()> {
        let server = task::spawn(async move {
            warp::serve(routes_handler(self.server_info)).run(self.socket).await;
        });

        server 
    }
}
