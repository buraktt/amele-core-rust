# amele_core

A Rust library for inter-process communication in the Amele workflow system.

## Overview

`amele_core` provides utilities for Amele worker processes to communicate with the orchestrator via TCP or shared memory. It handles message serialization using MessagePack and supports function call protocols.

## Features

- **TCP Communication** - Connect to the orchestrator via TCP socket
- **Shared Memory Support** - Read/write data using inbox/outbox files
- **MessagePack Serialization** - Efficient binary serialization with `rmp-serde`
- **Context Management** - Store and retrieve execution context
- **Function Calls** - Call remote functions and receive results

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
amele_core = { path = "../amele-core-rust" }
```

## Usage

```rust
use amele_core::{accept, context, call_function, complete};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Accept inputs from the orchestrator
    let inputs = accept()?;
    
    // Access execution context
    let ctx = context();
    
    // Call a remote function
    let mut call_inputs = HashMap::new();
    call_inputs.insert("key".to_string(), serde_json::json!("value"));
    let result = call_function("some_function", call_inputs)?;
    
    // Complete the task with outputs
    let mut outputs = HashMap::new();
    outputs.insert("result".to_string(), serde_json::json!("done"));
    complete(outputs)?;
    
    Ok(())
}
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `COMMUNICATION_PROTOCOL` | Set to `tcp` for TCP mode, otherwise uses shared memory |
| `AMELE_TCP_PORT` | TCP port for orchestrator connection |
| `AMELE_INBOX_FILE` | Path to inbox file (shared memory mode) |
| `AMELE_OUTBOX_FILE` | Path to outbox file (shared memory mode) |

## API

### `accept() -> Result<HashMap<String, Value>, Error>`

Initializes communication and receives input data from the orchestrator.

### `context() -> HashMap<String, Value>`

Returns the stored execution context.

### `call_function(name: &str, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, Error>`

Calls a remote function and returns the result.

### `complete(outputs: HashMap<String, Value>) -> Result<(), Error>`

Sends completion message with output data to the orchestrator.
