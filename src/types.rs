use std::collections::HashMap;

#[derive(Debug)]
pub enum TextOrNode {
	Text(String),
	Node(Node)
}

#[derive(Debug)]
pub struct Node {
	pub attributes: HashMap<String, String>,
	pub children: Vec<TextOrNode>,
	pub name: String
}

#[derive(Clone, Debug)]
pub struct TwoWay {
	ptr: usize,
	strm: Vec<char>
}

impl<'a> From<&'a str> for TwoWay {
	fn from(s: &'a str) -> TwoWay {
		TwoWay {
			strm: s.chars().collect(),
			ptr: 0
		}
	}
}

impl TwoWay {
	pub fn repeating(&self, start: usize, end: usize) -> bool {
		let s = self;
		let ptr = s.ptr() + start - end;
		!(0 .. end - start).into_iter().any(|i| s.strm[ptr + i] != s.strm[start + i])
	}
	
	pub fn debug(&self, n: usize) -> &[char] {
		&self.strm[n..]
	}
	
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
