use rum_core::Expr;
use rum_recursive_expr::Tree;
use rum_selective_sharing::{Probe, selective};
use rum_symbol_sharing::Stats;

#[derive(Clone, Copy)]
pub enum Policy {
    Plain,
    Shared,
    Selective,
}
impl Policy {
    pub fn name(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Shared => "shared",
            Self::Selective => "selective",
        }
    }
}
pub const POLICIES: [Policy; 3] = [Policy::Plain, Policy::Shared, Policy::Selective];
#[derive(Clone, Copy, Default)]
pub struct Record {
    pub probe: Option<Probe>,
    pub sharing: Option<Stats>,
}
pub fn build<N: Tree>(expr: &Expr, policy: Policy) -> (N, Record) {
    match policy {
        Policy::Plain => (rum_symbol_sharing::plain(expr), Record::default()),
        Policy::Shared => {
            let (node, stats) = rum_symbol_sharing::shared(expr, 128);
            (
                node,
                Record {
                    probe: None,
                    sharing: Some(stats),
                },
            )
        }
        Policy::Selective => {
            let (node, report) = selective(expr);
            (
                node,
                Record {
                    probe: Some(report.probe),
                    sharing: report.sharing,
                },
            )
        }
    }
}
#[cfg(feature = "allocation-count")]
impl Record {
    pub fn json(self) -> serde_json::Value {
        serde_json::json!({"probe": self.probe.map(|p| serde_json::json!({
            "decision": format!("{:?}",p.decision), "nodes":p.nodes,"symbols":p.symbols,"repeats":p.repeats})),
            "sharing": self.sharing.map(|s| serde_json::json!({"symbol_visits":s.symbol_visits,
                "hits":s.hits,"entries":s.entries,"uncached":s.uncached}))})
    }
}
