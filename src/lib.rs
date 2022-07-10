use svg::node::element::{SVG, Rectangle, Path, path};

pub struct SankeyStyle<F: FnMut(i32) -> String> {
	pub number_format: Option<F>,
	pub node_separation: Option<f64>,
	pub node_width: Option<f64>,
}

pub struct Sankey {
	nodes: Vec<SankeyNode>,
	edges: Vec<SankeyEdge>,
}

impl Sankey {
	pub fn new() -> Sankey {
		Sankey {
			nodes: Vec::new(),
			edges: Vec::new(),
		}
	}
	
	pub fn node(&mut self, value: Option<f64>, label: Option<String>, color: Option<String>) -> SankeyNodeID {
		let id = self.nodes.len();
		self.nodes.push(SankeyNode::new(value, label, color));
		SankeyNodeID(id)
	}
	
	pub fn edge(&mut self, source: SankeyNodeID, target: SankeyNodeID, value: f64, label: Option<String>, color: Option<String>) {
		self.edges.push(SankeyEdge { source, target, value, label, color });
		self.nodes[source.0].current_output += value;
		self.nodes[target.0].current_input += value;
	}
	
	pub fn value(&self, node: SankeyNodeID) -> Option<f64> {
		self.nodes[node.0].value
	}
	
	pub fn current_input(&self, node: SankeyNodeID) -> f64 {
		self.nodes[node.0].current_input
	}
	
	pub fn current_output(&self, node: SankeyNodeID) -> f64 {
		self.nodes[node.0].current_output
	}
	
	pub fn required_input(&self, node: SankeyNodeID) -> f64 {
		self.nodes[node.0].required_input()
	}
	
	pub fn required_output(&self, node: SankeyNodeID) -> f64 {
		self.nodes[node.0].required_output()
	}
	
	pub fn remaining_input(&self, node: SankeyNodeID) -> f64 {
		self.nodes[node.0].remaining_input()
	}
	
	pub fn remaining_output(&self, node: SankeyNodeID) -> f64 {
		self.nodes[node.0].remaining_output()
	}
	
	pub fn flow(&self, node: SankeyNodeID) -> f64 {
		self.nodes[node.0].flow()
	}
	
	pub fn draw<F: FnMut(i32) -> String>(&self, width: f64, height: f64, style: SankeyStyle<F>) -> SVG {
		let node_separation = style.node_separation.unwrap_or(height / 50.0);
		let node_width = style.node_width.unwrap_or(width / 100.0);
		
		
		// Initialise SVG
		
		let mut document =
			SVG::new()
			.set("viewBox", (0.0, 0.0, width, height));
		
		
		// Pre-process graph
		
		#[derive(Copy, Clone, Debug)]
		struct SankeyEdgeID(usize);
		
		#[derive(Clone, Debug)]
		struct Dependencies {
			inputs: Vec<SankeyEdgeID>,
			outputs: Vec<SankeyEdgeID>,
		}
		
		let mut dependency_counts = vec![0; self.nodes.len()];
		let mut node_edges = vec![Dependencies { inputs: Vec::new(), outputs: Vec::new() }; self.nodes.len()];
		for (id, &SankeyEdge { source, target, .. }) in self.edges.iter().enumerate() {
			node_edges[source.0].outputs.push(SankeyEdgeID(id));
			node_edges[target.0].inputs.push(SankeyEdgeID(id));
			dependency_counts[target.0] += 1;
		}
		let node_edges = node_edges;
		
		
		// Split into layers
		
		let mut layers = Vec::new();
		let mut next_layer = Vec::new();
		
		for (id, &count) in dependency_counts.iter().enumerate() {
			if count == 0 {
				next_layer.push(SankeyNodeID(id));
			}
		}
		
		let mut min_scale = f64::INFINITY;
		
		while !next_layer.is_empty() {
			layers.push(next_layer);
			next_layer = Vec::new();
			let current_layer = layers.last().unwrap();
			
			let mut total_value = 0.0;
			
			for node_id in current_layer {
				let node = &self.nodes[node_id.0];
				total_value += node.flow();
				for edge in &node_edges[node_id.0].outputs {
					let target = self.edges[edge.0].target;
					dependency_counts[target.0] -= 1;
					if dependency_counts[target.0] == 0 {
						next_layer.push(target);
					}
				}
			}
			
			let scale = (height - node_separation * ((current_layer.len() + 1) as f64)) / total_value;
			if scale < min_scale {
				min_scale = scale;
			}
		}
		
		
		// Draw nodes
		
		let mut positions = vec![(0.0, 0.0, 0.0); self.nodes.len()];
		
		let layer_width = (width - node_separation * 2.0 - (layers.len() as f64) * node_width) / ((layers.len() - 1) as f64);
		
		let mut x = node_separation;
		for layer in layers {
			let mut total_height = node_separation;
			for node_id in &layer {
				total_height += self.nodes[node_id.0].flow() * min_scale + node_separation;
			}
			let total_height = total_height;
			let mut y = node_separation + (height - total_height) / 2.0;
			for node_id in &layer {
				let node = &self.nodes[node_id.0];
				positions[node_id.0] = (x, y, y);
				document = document.add(
					Rectangle::new()
					.set("x", x)
					.set("y", y)
					.set("width", node_width)
					.set("height", node.flow() * min_scale)
					.set("fill", node.color.as_deref().unwrap_or("#000"))
				);
				y += node.flow() * min_scale + node_separation;
			}
			x += node_width + layer_width;
		}
		
		
		// Draw edges
		
		for edge in &self.edges {
			let thickness = edge.value * min_scale;
			let from_x = positions[edge.source.0].0 + node_width;
			let from_y_start = positions[edge.source.0].2;
			let from_y_end = from_y_start + thickness;
			let to_x = positions[edge.target.0].0;
			let to_y_start = positions[edge.target.0].1;
			let to_y_end = to_y_start + thickness;
			let mid_x = (from_x + to_x) / 2.0;
			//let mid_y = (from_y_start + to_y_end) / 2.0;
			
			positions[edge.source.0].2 = from_y_end;
			positions[edge.target.0].1 = to_y_end;
			
			document = document.add(
				Path::new()
				.set("d",
					path::Data::new()
					.move_to((from_x, from_y_start))
					.cubic_curve_to((mid_x, from_y_start, mid_x, to_y_start, to_x, to_y_start))
					.line_to((to_x, to_y_end))
					.cubic_curve_to((mid_x, to_y_end, mid_x, from_y_end, from_x, from_y_end))
					.close()
				)
				.set("fill", edge.color.as_deref().unwrap_or("#0005"))
			);
		}
		
		
		document
	}
}

pub struct SankeyNode {
	value: Option<f64>,
	label: Option<String>,
	color: Option<String>,
	current_input: f64,
	current_output: f64,
}

impl SankeyNode {
	pub fn new(value: Option<f64>, label: Option<String>, color: Option<String>) -> SankeyNode {
		SankeyNode {
			value,
			label,
			color,
			current_input: 0.0,
			current_output: 0.0,
		}
	}
	
	pub fn required_input(&self) -> f64 {
		self.value.unwrap_or(self.current_output)
	}
	
	pub fn required_output(&self) -> f64 {
		self.value.unwrap_or(self.current_input)
	}
	
	pub fn remaining_input(&self) -> f64 {
		self.required_input() - self.current_input
	}
	
	pub fn remaining_output(&self) -> f64 {
		self.required_output() - self.current_output
	}
	
	pub fn flow(&self) -> f64 {
		self.value.unwrap_or(f64::max(self.current_input, self.current_output))
	}
}

#[derive(Copy, Clone, Debug)]
pub struct SankeyNodeID(usize);

pub struct SankeyEdge {
	source: SankeyNodeID,
	target: SankeyNodeID,
	value: f64,
	label: Option<String>,
	color: Option<String>,
}
