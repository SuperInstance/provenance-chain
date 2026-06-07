//! # provenance-chain
//!
//! Knowledge provenance tracking with trust scoring.
//!
//! Track where knowledge came from, how it was derived, and compute
//! trust scores based on source reliability and chain depth.

/// A node tracking the provenance of a piece of knowledge.
#[derive(Debug, Clone, PartialEq)]
pub struct ProvenanceNode {
    pub id: String,
    pub who: String,
    pub when: f64,
    pub source: String,
    pub confidence: f64,
}

impl ProvenanceNode {
    pub fn new(
        id: impl Into<String>,
        who: impl Into<String>,
        when: f64,
        source: impl Into<String>,
        confidence: f64,
    ) -> Self {
        ProvenanceNode {
            id: id.into(),
            who: who.into(),
            when,
            source: source.into(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// A chain of provenance, linking knowledge back to its origin.
#[derive(Debug, Clone)]
pub struct ProvenanceChain {
    nodes: Vec<ProvenanceNode>,
}

impl ProvenanceChain {
    pub fn new() -> Self {
        ProvenanceChain { nodes: Vec::new() }
    }

    /// Add a provenance node to the chain.
    pub fn append(&mut self, node: ProvenanceNode) {
        self.nodes.push(node);
    }

    /// Prepend a provenance node (new origin).
    pub fn prepend(&mut self, node: ProvenanceNode) {
        self.nodes.insert(0, node);
    }

    /// Depth of the chain.
    pub fn depth(&self) -> usize {
        self.nodes.len()
    }

    /// Get the origin (first) node.
    pub fn origin(&self) -> Option<&ProvenanceNode> {
        self.nodes.first()
    }

    /// Get the latest (last) node.
    pub fn latest(&self) -> Option<&ProvenanceNode> {
        self.nodes.last()
    }

    /// Get all nodes.
    pub fn nodes(&self) -> &[ProvenanceNode] {
        &self.nodes
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Find a node by id.
    pub fn find(&self, id: &str) -> Option<&ProvenanceNode> {
        self.nodes.iter().find(|n| n.id == id)
    }
}

impl Default for ProvenanceChain {
    fn default() -> Self {
        Self::new()
    }
}

/// A trust score computed from provenance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrustScore(pub f64);

impl TrustScore {
    pub fn new(score: f64) -> Self {
        TrustScore(score.clamp(0.0, 1.0))
    }

    pub fn is_trusted(&self) -> bool {
        self.0 >= 0.7
    }

    pub fn is_suspicious(&self) -> bool {
        self.0 < 0.3
    }
}

/// Compute trust from chain depth and source reliability.
pub fn compute_trust(chain: &ProvenanceChain, reliability: &SourceReliability) -> TrustScore {
    if chain.is_empty() {
        return TrustScore::new(0.0);
    }
    // Base: average confidence of all nodes
    let avg_confidence: f64 = chain.nodes().iter().map(|n| n.confidence).sum::<f64>()
        / chain.depth() as f64;
    // Depth penalty: longer chains may introduce more uncertainty
    let depth_factor = 1.0 / (1.0 + 0.1 * (chain.depth().max(1) - 1) as f64);
    // Source reliability: average reliability of all sources in the chain
    let avg_reliability: f64 = chain
        .nodes()
        .iter()
        .map(|n| reliability.get(&n.source))
        .sum::<f64>()
        / chain.depth() as f64;
    TrustScore::new(avg_confidence * depth_factor * avg_reliability)
}

/// Track the reliability of knowledge sources over time.
#[derive(Debug, Clone)]
pub struct SourceReliability {
    scores: std::collections::HashMap<String, (f64, u32)>, // (total_score, count)
}

impl SourceReliability {
    pub fn new() -> Self {
        SourceReliability {
            scores: std::collections::HashMap::new(),
        }
    }

    /// Record an accuracy observation for a source.
    pub fn record(&mut self, source: &str, accuracy: f64) {
        let entry = self.scores.entry(source.to_string()).or_insert((0.0, 0));
        entry.0 += accuracy.clamp(0.0, 1.0);
        entry.1 += 1;
    }

    /// Get the average reliability for a source (defaults to 0.5).
    pub fn get(&self, source: &str) -> f64 {
        self.scores
            .get(source)
            .map(|(total, count)| total / *count as f64)
            .unwrap_or(0.5)
    }

    /// Get the number of observations for a source.
    pub fn observations(&self, source: &str) -> u32 {
        self.scores.get(source).map(|(_, c)| *c).unwrap_or(0)
    }
}

impl Default for SourceReliability {
    fn default() -> Self {
        Self::new()
    }
}

/// Query interface for finding origins of knowledge.
pub struct ProvenanceQuery<'a> {
    chain: &'a ProvenanceChain,
}

impl<'a> ProvenanceQuery<'a> {
    pub fn new(chain: &'a ProvenanceChain) -> Self {
        ProvenanceQuery { chain }
    }

    /// Find the original source of the knowledge.
    pub fn origin(&self) -> Option<&ProvenanceNode> {
        self.chain.origin()
    }

    /// Find all nodes from a specific source.
    pub fn from_source(&self, source: &str) -> Vec<&ProvenanceNode> {
        self.chain
            .nodes()
            .iter()
            .filter(|n| n.source == source)
            .collect()
    }

    /// Find all nodes by a specific agent.
    pub fn by_agent(&self, who: &str) -> Vec<&ProvenanceNode> {
        self.chain
            .nodes()
            .iter()
            .filter(|n| n.who == who)
            .collect()
    }

    /// Find nodes within a time range.
    pub fn in_time_range(&self, start: f64, end: f64) -> Vec<&ProvenanceNode> {
        self.chain
            .nodes()
            .iter()
            .filter(|n| n.when >= start && n.when <= end)
            .collect()
    }

    /// Trace the full lineage as a list of (who, source) pairs.
    pub fn lineage(&self) -> Vec<(&str, &str)> {
        self.chain
            .nodes()
            .iter()
            .map(|n| (n.who.as_str(), n.source.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_append() {
        let mut chain = ProvenanceChain::new();
        chain.append(ProvenanceNode::new("1", "agent_a", 1.0, "sensor", 0.9));
        chain.append(ProvenanceNode::new("2", "agent_b", 2.0, "derived", 0.8));
        assert_eq!(chain.depth(), 2);
    }

    #[test]
    fn test_chain_prepend() {
        let mut chain = ProvenanceChain::new();
        chain.append(ProvenanceNode::new("1", "a", 2.0, "s", 0.5));
        chain.prepend(ProvenanceNode::new("0", "root", 0.0, "original", 1.0));
        assert_eq!(chain.origin().unwrap().who, "root");
    }

    #[test]
    fn test_chain_origin_latest() {
        let mut chain = ProvenanceChain::new();
        chain.append(ProvenanceNode::new("1", "first", 1.0, "s1", 0.9));
        chain.append(ProvenanceNode::new("2", "second", 2.0, "s2", 0.7));
        assert_eq!(chain.origin().unwrap().who, "first");
        assert_eq!(chain.latest().unwrap().who, "second");
    }

    #[test]
    fn test_empty_chain() {
        let chain = ProvenanceChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.depth(), 0);
        assert!(chain.origin().is_none());
    }

    #[test]
    fn test_find_node() {
        let mut chain = ProvenanceChain::new();
        chain.append(ProvenanceNode::new("a1", "alice", 1.0, "sensor", 0.9));
        chain.append(ProvenanceNode::new("b1", "bob", 2.0, "api", 0.8));
        assert_eq!(chain.find("a1").unwrap().who, "alice");
        assert!(chain.find("c1").is_none());
    }

    #[test]
    fn test_trust_score() {
        let high = TrustScore::new(0.8);
        let low = TrustScore::new(0.2);
        assert!(high.is_trusted());
        assert!(!high.is_suspicious());
        assert!(!low.is_trusted());
        assert!(low.is_suspicious());
    }

    #[test]
    fn test_trust_clamped() {
        assert_eq!(TrustScore::new(1.5).0, 1.0);
        assert_eq!(TrustScore::new(-1.0).0, 0.0);
    }

    #[test]
    fn test_compute_trust_empty() {
        let chain = ProvenanceChain::new();
        let rel = SourceReliability::new();
        let trust = compute_trust(&chain, &rel);
        assert_eq!(trust.0, 0.0);
    }

    #[test]
    fn test_compute_trust_basic() {
        let mut chain = ProvenanceChain::new();
        chain.append(ProvenanceNode::new("1", "a", 1.0, "sensor", 0.9));
        let mut rel = SourceReliability::new();
        rel.record("sensor", 0.9);
        let trust = compute_trust(&chain, &rel);
        assert!(trust.0 > 0.5);
        assert!(trust.is_trusted());
    }

    #[test]
    fn test_source_reliability() {
        let mut rel = SourceReliability::new();
        rel.record("api", 0.8);
        rel.record("api", 1.0);
        assert!((rel.get("api") - 0.9).abs() < 1e-9);
        assert_eq!(rel.observations("api"), 2);
    }

    #[test]
    fn test_source_reliability_default() {
        let rel = SourceReliability::new();
        assert_eq!(rel.get("unknown"), 0.5);
        assert_eq!(rel.observations("unknown"), 0);
    }

    #[test]
    fn test_query_from_source() {
        let mut chain = ProvenanceChain::new();
        chain.append(ProvenanceNode::new("1", "a", 1.0, "sensor", 0.9));
        chain.append(ProvenanceNode::new("2", "b", 2.0, "api", 0.8));
        chain.append(ProvenanceNode::new("3", "c", 3.0, "sensor", 0.7));
        let query = ProvenanceQuery::new(&chain);
        assert_eq!(query.from_source("sensor").len(), 2);
        assert_eq!(query.from_source("api").len(), 1);
    }

    #[test]
    fn test_query_by_agent() {
        let mut chain = ProvenanceChain::new();
        chain.append(ProvenanceNode::new("1", "alice", 1.0, "s", 0.9));
        chain.append(ProvenanceNode::new("2", "bob", 2.0, "s", 0.8));
        let query = ProvenanceQuery::new(&chain);
        assert_eq!(query.by_agent("alice").len(), 1);
        assert_eq!(query.by_agent("charlie").len(), 0);
    }

    #[test]
    fn test_query_time_range() {
        let mut chain = ProvenanceChain::new();
        chain.append(ProvenanceNode::new("1", "a", 5.0, "s", 0.9));
        chain.append(ProvenanceNode::new("2", "b", 15.0, "s", 0.8));
        chain.append(ProvenanceNode::new("3", "c", 25.0, "s", 0.7));
        let query = ProvenanceQuery::new(&chain);
        assert_eq!(query.in_time_range(10.0, 20.0).len(), 1);
    }

    #[test]
    fn test_query_lineage() {
        let mut chain = ProvenanceChain::new();
        chain.append(ProvenanceNode::new("1", "alice", 1.0, "sensor", 0.9));
        chain.append(ProvenanceNode::new("2", "bob", 2.0, "api", 0.8));
        let query = ProvenanceQuery::new(&chain);
        let lineage = query.lineage();
        assert_eq!(lineage.len(), 2);
        assert_eq!(lineage[0], ("alice", "sensor"));
        assert_eq!(lineage[1], ("bob", "api"));
    }
}
