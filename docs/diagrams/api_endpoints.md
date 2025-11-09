# GATOS API Interaction Model

GATOS does not use a traditional RESTful API. Instead, it uses a JSONL (JSON Lines) RPC protocol for communication between clients (like SDKs or the CLI) and the `gatosd` daemon. Communication typically happens over `stdin`/`stdout` or a TCP socket.

This diagram illustrates the request/response flow for several key commands.

```mermaid
sequenceDiagram
    participant Client as Client (SDK/CLI)
    participant Daemon as gatosd

    Client->>Daemon: {"type":"append_event", "id":"01A", "ns":"...", "event":{...}}
    Daemon-->>Client: {"ok":true, "id":"01A", "commit_id":"..."}

    Client->>Daemon: {"type":"bus.publish", "id":"01B", "topic":"...", "payload":{...}}
    Daemon-->>Client: {"ok":true, "id":"01B", "msg_id":"..."}

    Client->>Daemon: {"type":"bus.subscribe", "id":"01C", "topic":"..."}
    Daemon-->>Client: {"ack":true, "id":"01C"}
    loop Subscription Stream
        Daemon-->>Client: {"type":"gmb.msg", "id":"01C", "topic":"...", "payload":{...}}
    end

    Client->>Daemon: {"type":"fold_state", "id":"01D", "ns":"..."}
    Daemon-->>Client: {"ok":true, "id":"01D", "state_root":"..."}
```
