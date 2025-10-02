use clap::Parser;

#[derive(Parser, Debug)]
/// Description: Task Solver creates a server with given amount
/// of workers. You can send create task request (POST) (on /create_task endpoint with json
/// {"type": "python/bin", "file": "...", "args": "..."})
/// with python scripts or base64 encoded
/// binary file to server to execute it. One of the free workers
/// will take this task and start subprocess.
/// You can check number of tasks in queue by sending GET request
/// on /get_task_count endpoint.
/// Also, you can check status of task by id, that was returned when you send
/// create task request. Send GET request with id in json in body of request.
pub struct ServerStartArguments {
    /// Amount of workers (tokio threads), that will be completing the tasks
    #[arg(short = 'w', long = "workers", default_value_t = 1)]
    pub workers_count: usize,
    /// Server address
    #[arg(short = 'a', long = "address", default_value = "127.0.0.1")]
    pub address: String,
    /// Server port
    #[arg(short = 'p', long = "port", default_value = "8080")]
    pub port: u16,
}
