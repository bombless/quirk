mod types;
mod parser;

fn attr_val() -> parser::ParseOperation<Result<String, ()>> {
    fn parse_quote(quote_char: char) -> parser::ParseOperation<Result<(((), String), ()), ()>> {
        let content = parser::any_char_except(vec![quote_char]);
        let start = parser::one_char(quote_char);
        let end = parser::one_char(quote_char);
        start + parser::take_while(content) + end
    }
    fn parse_naked() -> parser::ParseOperation<Result<String, ()>> {
        parser::take_while(parser::any_char_except(vec!['\n', '\r', '\t', ' ', '>', '/']))
    }
    let single_quote_form = parse_quote('\'');
    let double_quote_form = parse_quote('"');
    parse_naked() ^ parser::drop_first(parser::drop_second(single_quote_form ^ double_quote_form))
}

fn attr_name() -> parser::ParseOperation<Result<String, ()>> {
    parser::take_while(parser::any_char_except(vec!['\n', '\r', '\t', ' ', '>', '=']))
}

fn attr() -> parser::ParseOperation<Result<(String, String), ()>> {
    parser::Operation::from(Box::new(|s| {
        let name = skip_whitespaces(attr_name());
        let eq = skip_whitespaces(parser::one_char('='));
        let val = skip_whitespaces(attr_val());
        let ptr = s.ptr();
        match (name + eq + val).call(s) {
            Ok(((name, _), val)) => Ok((name, val)),
            _ => {
                s.set(ptr);
                Err(())
            }
        }
    }))
}

fn skip_whitespaces<R>(op: parser::ParseOperation<R>) -> parser::ParseOperation<R> {
    parser::Operation::from(Box::new(move |s| {
        while let Some(c) = s.read() {
            match c {
                ' ' | '\t' | '\n' | '\r' => (),
                _ => break
            }
        }
        let ptr = s.ptr();
        s.set(ptr - 1);
        op.call(s)
    }))
}

fn node_name() -> parser::ParseOperation<Result<String, ()>> {
    parser::take_while(parser::any_char_except(vec![' ', '\t', '\r', '\n', '>', '/']))
}

fn starting_tag() -> parser::ParseOperation<Result<(String, Vec<(String, String)>), ()>> {
    parser::drop_first(parser::one_char('<') + skip_whitespaces(node_name())) + parser::repeat_until(attr(), skip_whitespaces(parser::one_char('>')))
}

fn empty_node() -> parser::ParseOperation<Result<types::Node, ()>> {
    use std::collections::HashMap;
    let main = parser::one_char('<') + parser::twice(node_name(), parser::repeat_until(attr(), parser::one_char('>')) + parser::plain("</")) + parser::one_char('>');
    main.map(|((_, (name, (attrs, _))), _)| types::Node {
        attributes: attrs.into_iter().fold(HashMap::new(), |mut acc, x| {
            acc.insert(x.0, x.1);
            acc
        }),
        name: name,
        children: Vec::new()
    })
}

fn text_node() -> parser::ParseOperation<Result<types::TextOrNode, ()>> {
    parser::take_while(parser::any_char_except(vec!['<'])).map(types::TextOrNode::Text)
}

fn node() -> parser::ParseOperation<Result<types::Node, ()>> { // TODO support nested node
    use std::collections::HashMap;
    let starting = parser::one_char('<');
    let ending = parser::one_char('>');
    let text_node = text_node();
    let void_node = void_tag().map(types::TextOrNode::Node);
    let content = void_node ^ text_node;
    let whole_content = parser::repeat_until(content, parser::one_char('<') + skip_whitespaces(parser::one_char('/')) >> ());
    let attrs = parser::repeat_until(attr(), parser::one_char('>'));
    let main = starting + parser::twice(node_name(), attrs + whole_content) + ending;
    main.map(|((_, (name, (attrs, children))), _)| types::Node {
        attributes: attrs.into_iter().fold(HashMap::new(), |mut acc, x| {
            acc.insert(x.0, x.1);
            acc
        }),
        name: name,
        children: children
    })
}

fn void_tag() -> parser::ParseOperation<Result<types::Node, ()>> {
    fn map((name, attrs) : (String, Vec<(String, String)>)) -> types::Node {
        use std::collections::HashMap;
        types::Node {
            attributes: attrs.into_iter().fold(HashMap::new(), |mut acc, x| {
                acc.insert(x.0, x.1);
                acc
            }),
            name: name,
            children: Vec::new()
        }
    }
    let before_attrs = parser::drop_first(parser::one_char('<') + skip_whitespaces(node_name()));
    let after_attrs = parser::repeat_until(attr(), skip_whitespaces(parser::one_char('/')) + skip_whitespaces(parser::one_char('>')) >> ());
    (before_attrs + after_attrs).map(map)
}

fn main() {
    println!("{:?}", text_node().call(&mut "abc</tag>".into()));
    println!("{:?}", empty_node().call(&mut "<tag></tag>".into()));
    println!("{:?}", node().call(&mut "<tag some=1 hey=1><hey/>hey</tag>".into()));
}
