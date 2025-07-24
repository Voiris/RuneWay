pub struct SymbolTable {
    entries: Vec<String>
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable { entries: Vec::new() }
    }

    pub fn add(&mut self, name: String) {
        self.entries.push(name);
    }

    pub fn get(&self, id: usize) -> Option<&String> {
        self.entries.get(id)
    }

    pub fn get_by_entry(&self, symbol: &String) -> Option<usize> {
        self.entries.iter().position(|x| x == symbol)
    }
}
