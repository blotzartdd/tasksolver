# tasksolver
A task execution HTTP server that uses a limited number of threads to efficiently parallelize task processing.

The server distributes tasks from the common queue between worker processes,
each of which will execute the task on a separate thread and, upon completion of its execution, request the next one.

## Requests

### Creating tasks
Send an HTTP POST request to `/create_task` with a message body in the following format:

```json
{"type": "python/bin", "file": "...", "args": "..."}
```
where:
- `type` — the task type, either python for Python scripts or bin for binary files.
- `file`` — the Python program code or a binary file, encoded in base64.
- `args` — the arguments for program execution.

The server returns a JSON with the task identifier:
```json
{"id": "fb85a3a0-7e7f-4a20-8ced-65b3b2475144"}
```
This ID can be used to track the status and retrieve the task execution result.

### Retrieving Task Status

Clients can send an HTTP GET request to `/get_status` with the following body:

```json
{"id": "fb85a3a0-7e7f-4a20-8ced-65b3b2475144"}
```

where id is the task identifier obtained earlier. The server will return a response:
```json
{"status": "WAIT/RUNNING/SUCCESS/ERROR", "meta": {"created_at": "2024-11-10 00:00:00Z",
    "started_at": "2024-11-10 00:00:00Z", "finished_at": "2024-11-10 00:00:00Z"}, "result": {"stdout": "...", "stderr": "..."}}
```
- `status` — current task status: WAIT (in queue), RUNNING (executing), SUCCESS (completed successfully), ERROR (error).

- `meta` — nested JSON with information about task creation, start, and completion times.

- `created_at` — always present, indicates when the task was created.

- `started_at` — only present if task status is RUNNING, SUCCESS, or ERROR, indicates when the task was started.

- `finished_at` — only present if the task is completed, i.e., status is SUCCESS or ERROR.

- `result` — nested JSON with execution results, containing stdout for successful completion or stderr in case of error.

- `stdout` — contains task output upon successful execution.

- `stderr` — appears in addition to stdout if the task completed with an error.

### Retrieving Task Count Information

When sending an HTTP GET request to `/get_task_count`, the server returns the current number of tasks in the queue:

```json
{"tasks": 14}
```

## Running the project
When starting the server, specify three parameters: the number of worker threads, the address, and the port on which the server will listen for HTTP connections from clients:

```sh
cargo run -- --workers WORKERS_AMOUNT (default: 1) --address ADDRESS (default: 127.0.0.1) --port PORT (default: 8080)
```
