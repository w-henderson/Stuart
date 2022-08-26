use crate::parse::Token;

pub struct TokenIter<'a> {
    tokens: &'a [Token],
    index: usize,
}

pub struct TokenIterWaypoint(usize);

impl<'a> TokenIter<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, index: 0 }
    }

    pub fn waypoint(&self) -> TokenIterWaypoint {
        TokenIterWaypoint(self.index)
    }

    pub fn rewind_to(&mut self, waypoint: TokenIterWaypoint) {
        self.index = waypoint.0;
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.tokens.len() {
            let token = &self.tokens[self.index];
            self.index += 1;
            Some(token)
        } else {
            None
        }
    }
}
