mod types;
mod parser;


fn main() {
    let first = parser::comment(parser::one_char('"'), "parsing first quote");
    let second = parser::one_char('"');
    let empty_quote = first + second;
    let mut src = types::TwoWay::new(vec!['"', '"']);
    let mut from = src.clone();
    let result = empty_quote.call(&mut src);
    println!("{:?}", result);
    let result = parser::one_char('"') + parser::one_char('"').comment("parsing second quote") | &mut from;
    println!("{:?}", result);    
}
