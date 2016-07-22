use leema::val::{Val, Type};
use leema::lex::{lex};
use parse::{Parser, Token};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

/*
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum FuncType {
    Pure,
    //Query,
    //Cmd,
    //IO,
    //Sys,
}
*/

#[derive(Copy)]
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct TokenLoc
{
    lineno: i32,
    column: i16,
}

impl TokenLoc
{
    pub fn new(l: i32, c: i16) -> TokenLoc
    {
        TokenLoc{lineno: l, column: c}
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct TokenData<T>
{
    pub data: T,
    loc: TokenLoc,
}

impl<T> TokenData<T>
{
    pub fn new(d: T, tl: TokenLoc) -> TokenData<T>
    {
        TokenData{
            data: d,
            loc: tl,
        }
    }
}


// but shouldn't the Ast be regular s-exprs?
// but Rust has one, and it has macros
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Ast {
    ModuleRoot(Val),
    ReplRoot(Val),

    Nothing,
    Void,
}

impl Ast
{
    pub fn parse(toks: Vec<Token>) -> Ast {
        let e = Err(0);
        let mut p = Parser::new(e);
        for t in toks {
            p.parse(t);
        }
        p.parse(Token::EOI);
        p.into_extra().unwrap()
    }

    pub fn root(self) -> Val
    {
        match self {
            Ast::ModuleRoot(x) => x,
            Ast::ReplRoot(x) => x,
            _ => {
                panic!("not a root");
            }
        }
    }

}


pub trait Loader
{
    fn load(&self, filename: String) -> String;
    fn parse(&self, filename: String) -> Ast
    {
        let str = self.load(filename);
        let tokens = lex(str);
        Ast::parse(tokens)
    }
}

struct FileLoader
{
    path: Vec<String>,
}

pub fn new_file_loader() -> Box<Loader>
{
    Box::new(FileLoader{path: vec![]})
}

struct StringLoader
{
    files: HashMap<String, String>,
}

impl Loader for FileLoader
{
    fn load(&self, filename: String) -> String
    {
        let mut f: File = File::open(filename).unwrap();
        let mut input = String::new();
        f.read_to_string(&mut input).ok();
        //println!("file contents> {}", input);
        input
    }
}

impl Loader for StringLoader
{
    fn load(&self, filename: String) -> String
    {
        let result = self.files.get(&filename);
        if result.is_none() {
            panic!("String file cannot be imported: {}", filename);
        }
        result.unwrap().clone()
    }
}


#[cfg(test)]
mod tests {
    use leema::ast::{Ast};
    use leema::val::{Val, SexprType, Type};
    use leema::sexpr;
    use leema::list;
    use leema::lex::{lex};
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::sync::Arc;

#[test]
fn test_ast_parse_plus() {
    let input = "5 + 3\n".to_string();
    let lexed = lex(input);
    let root = Ast::parse(lexed);
    let xargs = list::cons(Val::Int(5), list::singleton(Val::Int(3)));
    let expected = Ast::ReplRoot(sexpr::new(SexprType::BlockExpr,
        list::singleton(sexpr::call("int_add".to_string(), xargs))
    ));
    assert_eq!(expected, root);
}

#[test]
fn test_ast_parse_strlit() {
    let input = "\"taco\"\n".to_string();
    let root = Ast::parse(lex(input));

    let expected = Ast::ReplRoot(sexpr::new(SexprType::BlockExpr,
        list::singleton(Val::new_str("taco".to_string()))
    ));
    assert_eq!(expected, root);
}

#[test]
fn test_ast_parse_string_id() {
    let input = "\"$var\"\n".to_string();
    let root = Ast::parse(lex(input));

    let expected = Ast::ReplRoot(sexpr::new(SexprType::BlockExpr,
        list::singleton(Val::id("var".to_string()))
    ));
    assert_eq!(expected, root);
}

#[test]
fn test_ast_parse_string_list() {
    let input = "\"hello $name\n\"\n".to_string();
    let root = Ast::parse(lex(input));

    let part1 = Val::new_str("hello ".to_string());
    let part2 = Val::id("name".to_string());
    let part3 = Val::new_str("\n".to_string());
    let expected = Ast::ReplRoot(
        sexpr::new(SexprType::BlockExpr, list::singleton(
            sexpr::new(SexprType::StrExpr,
                list::cons(part1,
                list::cons(part2,
                list::cons(part3,
                Val::Nil,
                ))))
        ))
    );
    assert_eq!(expected, root);
}

#[test]
fn test_ast_parse_plus_twice() {
    let input = "5 + 3 + 2\n".to_string();
    let root = Ast::parse(lex(input));

    let first_add = sexpr::call("int_add".to_string(),
        list::cons(Val::Int(5), list::singleton(Val::Int(3))),
    );
    let second_add = sexpr::call("int_add".to_string(),
        list::cons(first_add, list::singleton(Val::Int(2)))
    );

    let expected = Ast::ReplRoot(sexpr::new(SexprType::BlockExpr,
        list::singleton(second_add)
    ));
    assert_eq!(expected, root);
}

#[test]
fn test_ast_parse_call_one_param() {
    let input = "inc(-4)\n".to_string();
    let root = Ast::parse(lex(input));

    let neg4 = sexpr::call(
        "negate".to_string(),
        list::singleton(Val::Int(4)),
    );
    let expected = Ast::ReplRoot(sexpr::new(SexprType::BlockExpr,
        list::singleton(
            sexpr::call("inc".to_string(), list::singleton(neg4))
        )
    ));
    assert_eq!(expected, root);
}

#[test]
fn test_ast_parse_function_call() {
    let input = "foo(7, 2)\n".to_string();
    let root = Ast::parse(lex(input));

    let xargs = list::cons(Val::Int(7), list::singleton(Val::Int(2)));
    let expected = Ast::ReplRoot(sexpr::new(SexprType::BlockExpr,
        list::singleton(sexpr::call("foo".to_string(), xargs))
    ));
    assert_eq!(expected, root);
}

#[test]
fn test_ast_parse_tuple() {
    let input = "(3, \"taco\", true)\n".to_string();
    let root = Ast::parse(lex(input));

    let xtup = list::singleton(Val::Tuple(vec![
        Val::Int(3),
        Val::new_str("taco".to_string()),
        Val::Bool(true),
    ]));
    let expected = Ast::ReplRoot(sexpr::new(SexprType::BlockExpr, xtup));
    assert_eq!(expected, root);
}

#[test]
fn test_ast_parse_list() {
    let input = "[1, 2, x]\n".to_string();
    let root = Ast::parse(lex(input));

    let xlist = list::cons(Val::Int(1),
        list::cons(Val::Int(2),
        list::cons(Val::id("x".to_string()),
        Val::Nil,
    )));
    let expected = Ast::ReplRoot(sexpr::new(
        SexprType::BlockExpr, list::singleton(xlist)
    ));
    assert_eq!(expected, root);
}

#[test]
fn test_call_function_plus_comma()
{
    let input = "func main() {
        foo(x+1, 40)
    }
    ".to_string();
    Ast::parse(lex(input));
}

#[test]
fn test_call_function_comma_plus()
{
    let input = "func main() {
        foo(40, x+1)
    }
    ".to_string();
    Ast::parse(lex(input));
}

#[test]
fn test_parse_multiple_param_func()
{
    let input = "func doubles(x, x2) {
        x + x = x2
    }

    func main() {
        doubles(5, 10)
    }
    ".to_string();
    Ast::parse(lex(input));
}

#[test]
fn test_ast_parse_if()
{
    let input = "if x {
        y
    } else {
        z
    }
    ".to_string();
    let root = Ast::parse(lex(input));

    let blocka = sexpr::new(SexprType::BlockExpr, list::singleton(
        Val::id("y".to_string()),
    ));
    let blockb = sexpr::new(SexprType::BlockExpr, list::singleton(
        Val::id("z".to_string()),
    ));
    let expected = Ast::ReplRoot(sexpr::new(SexprType::BlockExpr, list::cons(
        sexpr::new(SexprType::IfExpr,
        list::cons(Val::id("x".to_string()),
        list::cons(blocka,
        list::cons(blockb,
        Val::Nil,
        )))),
        Val::Nil,
    )));
    assert_eq!(expected, root);
}

#[test]
fn test_ast_parse_macro()
{
    let input = "macro mand(a, b) {
        if a {
            b
        } else {
            false
        }
    }
    ".to_string();
    let root = Ast::parse(lex(input));

    let blocka = sexpr::new(SexprType::BlockExpr, list::singleton(
        Val::id("b".to_string()),
    ));
    let blockb = sexpr::new(SexprType::BlockExpr, list::singleton(
        Val::Bool(false)
    ));
    let ifx = sexpr::new(SexprType::BlockExpr, list::cons(
        sexpr::new(SexprType::IfExpr,
            list::cons(Val::id("a".to_string()),
            list::cons(blocka,
            list::cons(blockb,
            Val::Nil,
        )))),
        Val::Nil,
    ));
    let args =
        list::cons(Val::id("a".to_string()),
        list::cons(Val::id("b".to_string()),
        Val::Nil,
    ));
    let expected = Ast::ReplRoot(sexpr::new_block(list::singleton(
        sexpr::new(SexprType::DefMacro,
            list::cons(Val::id("mand".to_string()),
            list::cons(args,
            list::cons(ifx,
            Val::Nil,
        ))))
    )));
    assert_eq!(expected, root);
}

}
