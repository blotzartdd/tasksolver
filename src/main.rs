use clap::Parser;
use tasksolver::input_parser::ServerStartArguments;
use tasksolver::server::server::TaskSolverServer;

/// Runs tasksolver with arguments from command line
#[tokio::main]
async fn main() {
    let server_start_arguments = ServerStartArguments::parse();
    let tasksolver_server = TaskSolverServer::new(
        server_start_arguments.workers_count,
        server_start_arguments.address,
        server_start_arguments.port,
    );

    let tasksolver_handle = tasksolver_server.start_tasksolver_server().await;
    let _ = tasksolver_handle.await;
}
