#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    Function,
    Type,
    String,
    Number,
    Comment,
    Operator,
    Identifier,
    Constant,
    Macro,
    Property,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub start: usize, // character index
    pub end: usize,   // character index
    pub token_type: TokenType,
}
