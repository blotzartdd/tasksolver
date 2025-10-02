use crate::server::models::requests::{CreateTaskRequest, TaskType};
use crate::server::models::responses::TaskStatusEnum;
use base64::prelude::*;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Stdio;
use tokio::process::Command;

/// Creates temporary .bin file with permissions to open, write and execute it for everyone
fn create_temporary_binary_file(decoded_file: &Vec<u8>, id: &str) -> (String, String) {
    let path = format!("{}.bin", id);
    let mut temporary_file = File::create(path.clone()).unwrap();
    let _ = temporary_file.write_all(decoded_file.as_slice());

    let mut permissions = temporary_file.metadata().unwrap().permissions();
    permissions.set_mode(0o777);
    let _ = temporary_file.set_permissions(permissions);

    let execute_path = format!("./{}.bin", id);

    (path, execute_path)
}

/// Execute base64 encoded binary file (by creating temporary file with name of id)
/// and returns output
///
/// # Examples
///
/// use tasksolver::file_executer::file_executer::binary_execute;
/// use tasksolver::server::models::responses::TaskStatusEnum;
///
/// let id = "fb85a3a0-7e7f-4a20-8ced-65b3b2475144";
/// let base64_encoded_file = "ZWNobyBIZWxsbywgd29ybGQh"; // -> echo Hello, world!
/// let arguments = "Hello, world!".to_string();
///
/// let output = binary_execute(id, base64_encoded_file, arguments).await;
///
/// assert_eq!(output.stdout, "Hello, world!\n");
/// assert_eq!(output.stderr, None);
/// assert_eq!(output.status.success(), true);
///
pub async fn binary_execute(
    id: String,
    base64_encoded_file: String,
    arguments: String,
) -> std::process::Output {
    let decoded_file = BASE64_STANDARD.decode(base64_encoded_file).unwrap();
    let (temporary_file_path, execute_path) = create_temporary_binary_file(&decoded_file, &id);

    let output = Command::new(execute_path)
        .arg(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .unwrap();

    let _ = fs::remove_file(temporary_file_path);

    output
}

/// Execute python script and returns output
///
/// # Examples
///
/// use tasksolver::file_executer::file_executer::python_execute;
/// use crate::server::models::responses::TaskStatusEnum;
///
/// let python_code = "print(Hello, world!)";
/// let arguments = "".to_string();
///
/// let output = python_execute(python_code, arguments).await;
///
/// assert_eq!(output.stdout, "Hello, world!");
/// assert_eq!(output.stderr, None);
/// assert_eq!(output.status.success(), true);
pub async fn python_execute(python_code: String, arguments: String) -> std::process::Output {
    let output = Command::new("python3")
        .arg("-c")
        .arg(python_code)
        .arg(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .unwrap();

    output
}

/// Execute python script or binary file and returns stdout, stderr and task status
///
/// # Examples
///
/// use tasksolver::file_executer::file_executer::execute_file;
/// use crate::server::models::requests::{CreateTaskRequest, TaskType};
/// use crate::server::models::responses::TaskStatusEnum;
///
/// let id = "fb85a3a0-7e7f-4a20-8ced-65b3b2475144".to_string();
/// let base64_encoded_file = BASE64_STANDARD.encode("echo Hello, world!");
/// let arguments = "".to_string();
///
/// let create_task_request = CreateTaskRequest::new(TaskType::Bin, base64_encoded_file, arguments);
/// let (stdout, stderr, task_status) = execute_file(create_task_request, id).await;
///
/// assert_eq!(stdout, "Hello, world!");
/// assert_eq!(stderr, None);
/// assert_eq!(task_status, TaskStatusEnum::SUCCESS);
pub async fn execute_file(
    task: CreateTaskRequest,
    id: String,
) -> (String, Option<String>, TaskStatusEnum) {
    let task_type = task.task_type;
    let code = task.file;
    let arguments = task.args;

    let output = match task_type {
        TaskType::Python => python_execute(code, arguments).await,
        TaskType::Bin => binary_execute(id, code, arguments).await,
    };

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    if !output.status.success() {
        return (stdout, Some(stderr), TaskStatusEnum::ERROR);
    }

    (stdout, None, TaskStatusEnum::SUCCESS)
}

#[cfg(test)]
mod test_binary_execute {
    use crate::file_executer::file_executer::execute_file;
    use crate::server::models::requests::{CreateTaskRequest, TaskType};
    use crate::server::models::responses::TaskStatusEnum;
    use base64::prelude::*;

    #[tokio::test]
    async fn test_echo() {
        let id = "fb85a3a0-7e7f-4a20-8ced-65b3b2475144".to_string();
        let base64_encoded_file = BASE64_STANDARD.encode("echo Hello, world!");
        let arguments = "".to_string();

        let create_task_request =
            CreateTaskRequest::new(TaskType::Bin, base64_encoded_file, arguments);
        let (stdout, stderr, task_status) = execute_file(create_task_request, id).await;
        assert_eq!(stdout, "Hello, world!\n");
        assert_eq!(stderr, None);
        assert_eq!(task_status, TaskStatusEnum::SUCCESS);
    }

    #[tokio::test]
    async fn test_echo_with_special_symbol() {
        let id = "fb85a3a0-7e7f-4a20-8ced-65b3b2475145".to_string();
        let base64_encoded_file = BASE64_STANDARD.encode("echo Hello,\n world!");
        let arguments = "".to_string();

        let create_task_request =
            CreateTaskRequest::new(TaskType::Bin, base64_encoded_file, arguments);
        let (stdout, stderr, task_status) = execute_file(create_task_request, id).await;
        assert_eq!(stdout, "Hello,\n");
        assert_eq!(
            stderr,
            Some(
                "./fb85a3a0-7e7f-4a20-8ced-65b3b2475145.bin: line 2: world!: command not found\n"
                    .to_string()
            )
        );

        assert_eq!(task_status, TaskStatusEnum::ERROR);
    }

    #[tokio::test]
    async fn test_non_exist_command() {
        let id = "fb85a3a0-7e7f-4a20-8ced-65b3b2475145".to_string();
        let base64_encoded_file = BASE64_STANDARD.encode("bimbimbambam");
        let arguments = "".to_string();

        let create_task_request =
            CreateTaskRequest::new(TaskType::Bin, base64_encoded_file, arguments);
        let (_, _, task_status) = execute_file(create_task_request, id).await;
        assert_eq!(task_status, TaskStatusEnum::ERROR);
    }
}

#[cfg(test)]
mod test_python_execute {
    use crate::file_executer::file_executer::execute_file;
    use crate::server::models::requests::{CreateTaskRequest, TaskType};
    use crate::server::models::responses::TaskStatusEnum;

    #[tokio::test]
    async fn test_print() {
        let id = "fb85a3a0-7e7f-4a20-8ced-65b3b2475145".to_string();
        let python_code = "print('Hello, world!')".to_string();
        let arguments = "".to_string();

        let create_task_request = CreateTaskRequest::new(TaskType::Python, python_code, arguments);
        let (stdout, stderr, task_status) = execute_file(create_task_request, id).await;
        assert_eq!(stdout, "Hello, world!\n");
        assert_eq!(stderr, None);
        assert_eq!(task_status, TaskStatusEnum::SUCCESS);
    }

    #[tokio::test]
    async fn test_cycle() {
        let id = "fb85a3a0-7e7f-4a20-8ced-65b3b2475145".to_string();
        let python_code = "for i in range(5):
                            print(i)"
            .to_string();
        let arguments = "".to_string();

        let create_task_request = CreateTaskRequest::new(TaskType::Python, python_code, arguments);
        let (stdout, stderr, task_status) = execute_file(create_task_request, id).await;
        assert_eq!(stdout, "0\n1\n2\n3\n4\n");
        assert_eq!(stderr, None);
        assert_eq!(task_status, TaskStatusEnum::SUCCESS);
    }

    #[tokio::test]
    async fn test_zero_division_error() {
        let id = "fb85a3a0-7e7f-4a20-8ced-65b3b2475145".to_string();
        let python_code = "print(1 / 0)".to_string();
        let arguments = "".to_string();

        let create_task_request = CreateTaskRequest::new(TaskType::Python, python_code, arguments);
        let (stdout, stderr, task_status) = execute_file(create_task_request, id).await;
        assert_eq!(stdout, "");
        assert_eq!(stderr, Some("Traceback (most recent call last):\n  File \"<string>\", line 1, in <module>\nZeroDivisionError: division by zero\n".to_string()));
        assert_eq!(task_status, TaskStatusEnum::ERROR);
    }

    #[tokio::test]
    async fn test_arguments() {
        let id = "fb85a3a0-7e7f-4a20-8ced-65b3b2475145".to_string();
        let python_code = "import sys

print(sys.argv[1])"
            .to_string();
        let arguments = "test_argument".to_string();

        let create_task_request = CreateTaskRequest::new(TaskType::Python, python_code, arguments);
        let (stdout, stderr, task_status) = execute_file(create_task_request, id).await;
        assert_eq!(stdout, "test_argument\n");
        assert_eq!(stderr, None);
        assert_eq!(task_status, TaskStatusEnum::SUCCESS);
    }
}
