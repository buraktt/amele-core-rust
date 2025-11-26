use rmp_serde;
use serde_json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Mutex, OnceLock};

static STREAM: OnceLock<Mutex<TcpStream>> = OnceLock::new();
static STORED_CONTEXT: OnceLock<Mutex<HashMap<String, serde_json::Value>>> = OnceLock::new();

pub fn accept() -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error>> {
    if env::var("COMMUNICATION_PROTOCOL").unwrap_or_default() == "tcp" {
        let port = env::var("AMELE_TCP_PORT")?;
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
        let _ = STREAM.set(Mutex::new(stream));

        let mut stream_lock = STREAM.get().unwrap().lock().unwrap();
        let envelope: HashMap<String, serde_json::Value> = rmp_serde::from_read(&mut *stream_lock)?;

        // Extract context and inputs from envelope
        let ctx = envelope
            .get("context")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(HashMap::new);
        let _ = STORED_CONTEXT.set(Mutex::new(ctx));

        let inputs = envelope
            .get("inputs")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(HashMap::new);
        return Ok(inputs);
    }

    // For shmem mode, read from inbox file
    let inbox_path = env::var("AMELE_INBOX_FILE").ok();
    if let Some(path) = inbox_path {
        let data = fs::read(path)?;
        let envelope: HashMap<String, serde_json::Value> = rmp_serde::from_slice(&data)?;

        // Extract context and inputs from envelope
        let ctx = envelope
            .get("context")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(HashMap::new);
        let _ = STORED_CONTEXT.set(Mutex::new(ctx));

        let inputs = envelope
            .get("inputs")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(HashMap::new);
        return Ok(inputs);
    }

    let _ = STORED_CONTEXT.set(Mutex::new(HashMap::new()));
    Ok(HashMap::new())
}

pub fn context() -> HashMap<String, serde_json::Value> {
    STORED_CONTEXT
        .get()
        .map(|m| m.lock().unwrap().clone())
        .unwrap_or_else(HashMap::new)
}

pub fn call_function(
    function_name: &str,
    inputs: HashMap<String, serde_json::Value>,
) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error>> {
    if env::var("COMMUNICATION_PROTOCOL").unwrap_or_default() == "tcp" {
        let mut stream_lock = STREAM
            .get()
            .ok_or("Stream not initialized")?
            .lock()
            .unwrap();

        let id = uuid::Uuid::new_v4().to_string();
        let req = serde_json::json!({
            "type": "call",
            "function": function_name,
            "inputs": inputs,
            "id": id
        });

        let encoded = rmp_serde::to_vec(&req)?;
        stream_lock.write_all(&encoded)?;

        let resp: serde_json::Value = rmp_serde::from_read(&mut *stream_lock)?;

        if resp["type"] == "call_result" && resp["id"] == id {
            if let Some(err) = resp.get("error") {
                return Err(err.as_str().unwrap_or("Unknown error").into());
            }
            if let Some(result) = resp.get("result") {
                return Ok(serde_json::from_value(result.clone())?);
            }
            return Ok(HashMap::new());
        }
        return Err(format!("Unexpected response: {:?}", resp).into());
    }
    Err("callFunction is not supported in shmem mode".into())
}

pub fn respond(
    context: HashMap<String, serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    if env::var("COMMUNICATION_PROTOCOL").unwrap_or_default() == "tcp" {
        let mut stream_lock = STREAM
            .get()
            .ok_or("Stream not initialized")?
            .lock()
            .unwrap();
        let msg = serde_json::json!({
            "type": "respond",
            "context": context
        });
        let encoded = rmp_serde::to_vec(&msg)?;
        stream_lock.write_all(&encoded)?;
        return Ok(());
    }

    let shmem_path = env::var("AMELE_OUTBOX_FILE")?;
    let updated_data = rmp_serde::to_vec(&context)?;
    fs::write(shmem_path, updated_data)?;
    Ok(())
}
