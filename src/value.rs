#[derive(Debug)]
pub enum Value {
    String(String),
    Int(i64),
    Bool(bool),
    Null,
}
