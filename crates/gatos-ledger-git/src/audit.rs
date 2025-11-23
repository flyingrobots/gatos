use gatos_ports::{AuditError, AuditSink, PolicyAuditEntry};
use git2::{Repository, Signature};
use serde::Serialize;

/// Git-backed audit sink writing policy decisions under
/// `refs/gatos/audit/policy/<ns>/<actor>`.
pub struct GitPolicyAudit<'r> {
    repo: &'r Repository,
    ns: String,
    actor: String,
}

impl<'r> GitPolicyAudit<'r> {
    pub fn new(repo: &'r Repository, ns: &str, actor: &str) -> Self {
        Self {
            repo,
            ns: ns.to_string(),
            actor: actor.to_string(),
        }
    }

    fn refname(&self) -> String {
        format!("refs/gatos/audit/policy/{}/{}", self.ns, self.actor)
    }
}

impl AuditSink for GitPolicyAudit<'_> {
    fn record_policy_decision(&self, entry: &PolicyAuditEntry) -> Result<(), AuditError> {
        let sig =
            Signature::now("gatos-ledger", "ledger@gatos.local").map_err(|_| AuditError::Io)?;
        let refname = self.refname();

        #[derive(Serialize)]
        struct AuditDoc<'a> {
            timestamp: u64,
            decision: &'a gatos_ports::PolicyDecision,
            ctx: &'a gatos_ports::AppendContext,
        }
        let doc = AuditDoc {
            timestamp: entry.timestamp,
            decision: &entry.decision,
            ctx: &entry.ctx,
        };
        let bytes = serde_json::to_vec(&doc).map_err(|_| AuditError::Other("serde".into()))?;
        let blob = self.repo.blob(&bytes).map_err(|_| AuditError::Io)?;
        let mut tb = self.repo.treebuilder(None).map_err(|_| AuditError::Io)?;
        tb.insert("audit.json", blob, 0o100644)
            .map_err(|_| AuditError::Io)?;
        let tree_oid = tb.write().map_err(|_| AuditError::Io)?;
        let tree = self.repo.find_tree(tree_oid).map_err(|_| AuditError::Io)?;

        // CAS update: read current, then reference_matching.
        let expected = self
            .repo
            .find_reference(&refname)
            .ok()
            .and_then(|r| r.target());

        let msg = format!("policy:{:?}", entry.decision.outcome);
        let commit_oid = self
            .repo
            .commit(None, &sig, &sig, &msg, &tree, &[])
            .map_err(|_| AuditError::Io)?;

        let res = if let Some(expected) = expected {
            self.repo
                .reference_matching(&refname, commit_oid, true, expected, "policy audit")
        } else {
            self.repo
                .reference(&refname, commit_oid, true, "policy audit")
        };

        match res {
            Ok(_) => Ok(()),
            Err(e)
                if e.class() == git2::ErrorClass::Reference
                    && (e.code() == git2::ErrorCode::Locked
                        || e.code() == git2::ErrorCode::NotFound) =>
            {
                Err(AuditError::Conflict)
            }
            Err(_) => Err(AuditError::Io),
        }
    }
}
