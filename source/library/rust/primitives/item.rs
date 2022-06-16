use super::BodyId;
use super::BodySymbolRef;
use super::GrammarStore;
use super::ProductionId;
use super::SymbolID;

/// Represents a specific point in a parse sequence
/// defined by a body and a positional offset that
/// indicates the next expected terminal or non-terminal.
#[repr(C, align(64))]
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]

pub struct Item
{
    body:   BodyId,
    length: u8,
    offset: u8,
    state:  u32,
}

impl Item
{
    pub fn debug_string(&self, grammar: &GrammarStore) -> String
    {
        let body = grammar.bodies_table.get(&self.body).unwrap();

        let mut string = String::new();

        string += &format!("[ {} ]", self.state);

        string += &grammar.production_table.get(&body.production).unwrap().name;

        string += " =>";

        for (index, BodySymbolRef { sym_id, .. }) in
            body.symbols.iter().enumerate()
        {
            if index == self.offset as usize {
                string += " •";
            }

            string += " ";

            string += &sym_id.to_string(grammar)
        }

        if self.at_end() {
            string += " •";
        }

        string
    }

    /// Create an Item from a body_id and a grammar store. Returns
    /// None if the body_id does not match a stored body in the
    /// grammar.

    pub fn from_body(body_id: &BodyId, grammar: &GrammarStore) -> Option<Self>
    {
        match grammar.bodies_table.get(&body_id) {
            Some(body) => Some(Item {
                body:   *body_id,
                length: body.length as u8,
                offset: 0,
                state:  0,
            }),
            _ => None,
        }
    }

    pub fn at_end(&self) -> bool
    {
        return self.offset == self.length;
    }

    pub fn to_state(&self, state: u32) -> Item
    {
        Item {
            length: self.length,
            offset: self.offset,
            body: self.body,
            state,
        }
    }

    pub fn to_last_sym(self) -> Self
    {
        Item {
            body:   self.body,
            length: self.length,
            offset: self.length - 1,
            state:  self.state,
        }
    }

    pub fn to_start(&self) -> Item
    {
        Item {
            body:   self.body,
            length: self.length,
            offset: 0,
            state:  self.state,
        }
    }

    pub fn to_end(&self) -> Item
    {
        Item {
            body:   self.body,
            length: self.length,
            offset: self.length,
            state:  self.state,
        }
    }

    pub fn to_zero_state(&self) -> Item
    {
        Item {
            body:   self.body,
            length: self.length,
            offset: self.offset,
            state:  0,
        }
    }

    pub fn increment(&self) -> Option<Item>
    {
        if !self.at_end() {
            Some(Item {
                length: self.length,
                offset: self.offset + 1,
                body:   self.body,
                state:  self.state,
            })
        } else {
            None
        }
    }

    pub fn decrement(&self) -> Option<Item>
    {
        if !self.is_start() {
            Some(Item {
                length: self.length,
                offset: self.offset - 1,
                body:   self.body,
                state:  self.state,
            })
        } else {
            None
        }
    }

    pub fn is_start(&self) -> bool
    {
        self.offset == 0
    }

    pub fn get_body(&self) -> BodyId
    {
        self.body
    }

    pub fn get_offset(&self) -> u32
    {
        self.offset as u32
    }

    pub fn get_state(&self) -> u32
    {
        self.state as u32
    }

    pub fn get_length(&self) -> u32
    {
        self.length as u32
    }

    pub fn get_hash(&self) -> u64
    {
        let body_id = self.body.0;

        (body_id & 0xFFFFFFFF_FFFFFF00) | (self.offset as u64)
    }

    pub fn get_hash_with_state(&self) -> u64
    {
        let hash = self.get_hash();

        (hash & 0xFFFFFFFF_000000FF) | (self.state << 8) as u64
    }

    pub fn get_symbol(&self, grammar: &GrammarStore) -> SymbolID
    {
        if self.at_end() {
            SymbolID::EndOfFile
        } else {
            match grammar.bodies_table.get(&self.body) {
                Some(body) => body.symbols[self.offset as usize].sym_id,
                _ => SymbolID::Undefined,
            }
        }
    }

    pub fn get_production_id_at_sym(
        &self,
        grammar: &GrammarStore,
    ) -> ProductionId
    {
        match self.get_symbol(grammar) {
            SymbolID::Production(production, _) => production,
            _ => ProductionId(0),
        }
    }

    pub fn is_end(&self) -> bool
    {
        self.length <= self.offset
    }

    pub fn is_term(&self, grammar: &GrammarStore) -> bool
    {
        if self.is_end() {
            false
        } else {
            match self.get_symbol(grammar) {
                SymbolID::Production(production, _) => false,
                _ => true,
            }
        }
    }

    pub fn is_nonterm(&self, grammar: &GrammarStore) -> bool
    {
        if self.is_end() {
            false
        } else {
            !self.is_term(grammar)
        }
    }

    pub fn get_production_id(&self, grammar: &GrammarStore) -> ProductionId
    {
        grammar
            .bodies_table
            .get(&self.get_body())
            .unwrap()
            .production
    }
}
