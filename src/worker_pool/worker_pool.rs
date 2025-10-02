use crate::file_executer::file_executer::execute_file;
use crate::server::models::requests::CreateTaskRequest;
use crate::server::server::TaskStatus;

use tokio::task;

/// Struct of all needed task information
pub struct TaskInfo {
    pub id: String,
    pub task_request: CreateTaskRequest,
    pub task_status: TaskStatus,
}

impl TaskInfo {
    /// Creates new task info struct from id, request and status concurrent hashmap
    pub fn new(id: String, task_request: CreateTaskRequest, task_status: TaskStatus) -> TaskInfo {
        TaskInfo {
            id,
            task_request,
            task_status,
        }
    }
}

/// Struct of tokio threads that will be taking tasks from sender and execute them.
pub struct WorkerPool {
    /// Amount of threads in thread pool
    workers_count: usize,
    /// Tokio sender that sends task id, task request, task status hashmap and current worker pool
    pub sender: async_channel::Sender<TaskInfo>,
    /// Tokio receiver that recieve task id, task request, task status hashmap and current worker pool
    pub receiver: async_channel::Receiver<TaskInfo>,
}

impl WorkerPool {
    /// Creates WorkerPool struct with given amount of workes, sender and receiver.
    pub fn new(
        workers_count: usize,
        sender: async_channel::Sender<TaskInfo>,
        receiver: async_channel::Receiver<TaskInfo>,
    ) -> WorkerPool {
        for _ in 0..workers_count {
            create_worker(receiver.clone());
        }

        WorkerPool {
            workers_count,
            sender,
            receiver,
        }
    }

    /// Increases amount of currently working threads and send task in receiver
    /// for free thread to pick up it
    pub async fn do_task(&self, task_info: TaskInfo) {
        let _ = self.sender.send(task_info).await;
    }

    pub fn get_task_amount(&self) -> usize {
        self.sender.len()
    }
}

/// Creates tokio thread that will execute python scripts and binary files
fn create_worker(receiver: async_channel::Receiver<TaskInfo>) {
    task::spawn(async move {
        loop {
            if let Ok(task_info) = receiver.recv().await {
                let mut task_status = task_info.task_status;
                task_status.start_running_task(&task_info.id);

                let (stdout, stderr, execution_result) =
                    execute_file(task_info.task_request, task_info.id.clone()).await;

                task_status.finish_running_task(&task_info.id, stdout, stderr, execution_result);
            }
        }
    });
}
