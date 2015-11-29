use std::collections::HashMap;

pub enum TextOrNode {
	Text(String),
	Node(Node)
}

pub struct Node {
	attributes: HashMap<String, String>,
	children: Vec<TextOrNode>,
	name: String
}

pub struct TwoWay {
	ptr: usize,
	strm: Vec<char>
}

impl TwoWay {
	pub fn new(v: Vec<char>) -> TwoWay {
		TwoWay { strm: v, ptr: 0 }
	}
	
	pub fn ptr(&self) -> usize {
		self.ptr
	}
	
	pub fn set(&mut self, ptr: usize) {
		self.ptr = ptr
	}
	
	pub fn read(&mut self) -> Option<char> {
		if self.ptr < self.strm.len() {
			self.ptr += 1;
			Some(self.strm[self.ptr - 1])
		} else {
			None
		}
	}
}