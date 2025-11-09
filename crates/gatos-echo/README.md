# GATOS Echo (State Engine)

This crate contains the deterministic state engine for GATOS. It is responsible for managing sessions (ephemeral branches) and applying "folds" to the event journal to produce deterministic state. It uses a DPO (Double-Pushout) graph engine to handle state transitions and merges.

For more details on the overall architecture, see the main [GATOS Technical Specification](../../docs/TECH-SPEC.md).
