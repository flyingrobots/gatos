# GATOS Mind (Message Bus)

This crate implements the GATOS Message Bus (GMB), an asynchronous, commit-backed publish/subscribe system. It handles topics, sharding, and different Quality of Service (QoS) guarantees for distributed communication between GATOS components.

Commit-backed means messages are persisted as Git commits to provide durability, auditability, and exactly-once semantics when combined with acknowledgements/commitments. See the architecture notes in [ADR-0001](../../docs/decisions/ADR-0001/DECISION.md) and the protocol details in [TECH-SPEC.md](../../docs/TECH-SPEC.md).

For more details on the overall architecture, see the main [GATOS Technical Specification](../../docs/TECH-SPEC.md).
