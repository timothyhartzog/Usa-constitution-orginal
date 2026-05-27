use constitution_archive::CitationGraphView;

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub key: String,
    pub kind: String,
    #[allow(dead_code)]
    pub citation_count: usize,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub radius: f64,
}

#[derive(Debug, Clone)]
pub struct LayoutEdge {
    pub source: usize,
    pub target: usize,
    pub weight: usize,
}

pub struct ForceGraph {
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<LayoutEdge>,
    pub width: f64,
    pub height: f64,
}

impl ForceGraph {
    pub fn from_view(view: &CitationGraphView, width: f64, height: f64) -> Self {
        let cx = width / 2.0;
        let cy = height / 2.0;
        let n = view.nodes.len();
        let ring_r = f64::min(width, height) * 0.28;

        let max_count = view
            .nodes
            .iter()
            .map(|n| n.citation_count)
            .max()
            .unwrap_or(1) as f64;

        let nodes: Vec<LayoutNode> = view
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n.max(1) as f64);
                let r = 4.0 + 18.0 * (node.citation_count as f64 / max_count).sqrt();
                LayoutNode {
                    key: node.key.clone(),
                    kind: node.kind.clone(),
                    citation_count: node.citation_count,
                    x: cx + ring_r * angle.cos(),
                    y: cy + ring_r * angle.sin(),
                    vx: 0.0,
                    vy: 0.0,
                    radius: r,
                }
            })
            .collect();

        let key_to_idx: std::collections::HashMap<&str, usize> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.key.as_str(), i))
            .collect();

        let edges: Vec<LayoutEdge> = view
            .edges
            .iter()
            .filter_map(|e| {
                let s = key_to_idx.get(e.source.as_str())?;
                let t = key_to_idx.get(e.target.as_str())?;
                Some(LayoutEdge {
                    source: *s,
                    target: *t,
                    weight: e.weight as usize,
                })
            })
            .collect();

        let mut graph = Self {
            nodes,
            edges,
            width,
            height,
        };
        graph.simulate(150);
        graph
    }

    pub fn simulate(&mut self, ticks: usize) {
        let repulsion = 800.0;
        let attraction = 0.005;
        let damping = 0.85;
        let cx = self.width / 2.0;
        let cy = self.height / 2.0;
        let center_pull = 0.01;

        for _ in 0..ticks {
            let n = self.nodes.len();

            // Repulsion between all pairs
            for i in 0..n {
                for j in (i + 1)..n {
                    let dx = self.nodes[i].x - self.nodes[j].x;
                    let dy = self.nodes[i].y - self.nodes[j].y;
                    let dist_sq = dx * dx + dy * dy + 1.0;
                    let force = repulsion / dist_sq;
                    let fx = dx * force / dist_sq.sqrt();
                    let fy = dy * force / dist_sq.sqrt();
                    self.nodes[i].vx += fx;
                    self.nodes[i].vy += fy;
                    self.nodes[j].vx -= fx;
                    self.nodes[j].vy -= fy;
                }
            }

            // Attraction along edges
            for edge in &self.edges {
                let dx = self.nodes[edge.target].x - self.nodes[edge.source].x;
                let dy = self.nodes[edge.target].y - self.nodes[edge.source].y;
                let force = attraction * edge.weight as f64;
                let fx = dx * force;
                let fy = dy * force;
                self.nodes[edge.source].vx += fx;
                self.nodes[edge.source].vy += fy;
                self.nodes[edge.target].vx -= fx;
                self.nodes[edge.target].vy -= fy;
            }

            // Center gravity
            for node in &mut self.nodes {
                node.vx += (cx - node.x) * center_pull;
                node.vy += (cy - node.y) * center_pull;
            }

            // Apply velocities with damping
            let margin = 20.0;
            for node in &mut self.nodes {
                node.vx *= damping;
                node.vy *= damping;
                node.x += node.vx;
                node.y += node.vy;
                node.x = node.x.clamp(margin, self.width - margin);
                node.y = node.y.clamp(margin, self.height - margin);
            }
        }
    }
}
