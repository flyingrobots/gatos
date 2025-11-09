# GATOS State Management Example

This diagram illustrates how the state of an entity (in this case, a Job) transitions based on events recorded in the GATOS ledger. This is a conceptual model; the actual state is derived by a "fold" process running in `gatos-echo`.

```mermaid
stateDiagram-v2
    [*] --> Enqueued

    Enqueued --> Processing: Worker consumes `gmb.msg`
    Processing --> Succeeded: `jobs.result` (ok) event recorded
    Processing --> Failed: `jobs.result` (fail) event recorded

    Succeeded --> [*]
    Failed --> Retrying: `attempts` < max_retries
    Failed --> DeadLetterQueue: `attempts` >= max_retries

    Retrying --> Enqueued: Job is re-published
    DeadLetterQueue --> [*]
```
