#[derive(Debug)]
pub struct Context {
    pub depth: Depth,
    pub fully_specified: Bool,
    pub resolve_to_context: Bool,
}

impl Context {
    pub fn new(fully_specified: bool, resolve_to_context: bool) -> Self {
        Self {
            depth: Depth::new(),
            fully_specified: Bool(fully_specified),
            resolve_to_context: Bool(resolve_to_context),
        }
    }
}

#[derive(Debug)]
pub struct Bool(bool);

impl Bool {
    pub fn set(&mut self, value: bool) {
        self.0 = value
    }
    pub fn get(&self) -> bool {
        self.0
    }
}

#[derive(Debug)]
pub struct Depth(u16);

impl Depth {
    fn new() -> Self {
        Self(0)
    }

    pub fn increase(&mut self) {
        self.0 += 1;
    }

    pub fn decrease(&mut self) {
        self.0 -= 1;
    }

    pub fn cmp(&self, other: u16) -> std::cmp::Ordering {
        self.0.cmp(&other)
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}
