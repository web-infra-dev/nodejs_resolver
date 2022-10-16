#[derive(Debug)]
pub struct Context {
    pub depth: Depth,
}

impl Context {
    pub fn new() -> Self {
        Self {
            depth: Depth::new(),
        }
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
}
