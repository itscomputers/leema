use leema::lri::Lri;
use leema::lstr::Lstr;
use leema::parse::{Parser, Token};
use leema::val::SrcLoc;
use leema::val::Type;

use std::collections::LinkedList;

/*
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum FuncClass {
    Pure,
    //Query,
    //Cmd,
    //IO,
    //Sys,
}
*/


#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct TokenData<T>
{
    pub data: T,
    pub loc: SrcLoc,
}

impl<T> TokenData<T>
{
    pub fn new(d: T, tl: SrcLoc) -> TokenData<T>
    {
        TokenData { data: d, loc: tl }
    }
}


#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
pub enum LetType
{
    Forked,
    Inline,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
pub enum DataType
{
    Enum,
    Struple,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
pub enum FuncClass
{
    Macro,
    Func,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
pub enum IfType
{
    If,
    Match,
    MatchFailure,
    TypeCast,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct IfCase
{
    pub cond: Ast,
    pub body: Ast,
    pub else_case: Option<Box<IfCase>>,
    pub loc: SrcLoc,
}

impl IfCase
{
    pub fn new(
        cond: Ast,
        body: Ast,
        else_case: Option<IfCase>,
        loc: SrcLoc,
    ) -> IfCase
    {
        IfCase {
            cond,
            body,
            else_case: else_case.map(|ic| Box::new(ic)),
            loc,
        }
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct Kxpr
{
    k: Option<Lstr>,
    x: Option<Box<Ast>>,
}

impl Kxpr
{
    pub fn new(k: Lstr, x: Ast) -> Kxpr
    {
        Kxpr {
            k: Some(k),
            x: Some(Box::new(x)),
        }
    }

    pub fn new_k(k: Lstr) -> Kxpr
    {
        Kxpr {
            k: Some(k),
            x: None,
        }
    }

    pub fn new_x(x: Ast) -> Kxpr
    {
        Kxpr {
            k: None,
            x: Some(Box::new(x)),
        }
    }

    pub fn k_ref(&self) -> Option<&Lstr>
    {
        self.k.as_ref()
    }

    pub fn x_ref(&self) -> Option<&Ast>
    {
        self.x.as_ref().map(|x| &**x)
    }

    pub fn k_clone(&self) -> Option<Lstr>
    {
        self.k.clone()
    }

    pub fn map_x<F>(&self, op: F) -> Kxpr
    where
        F: FnOnce(&Ast) -> Ast,
    {
        let new_x = self.x.as_ref().map(|bx| Box::new(op(&**bx)));
        Kxpr {
            k: self.k.clone(),
            x: new_x,
        }
    }

    pub fn map_1<F, T>(&self, op: F) -> (Option<Lstr>, T)
    where
        F: FnOnce(&Ast) -> T,
    {
        let x_u = self.x.as_ref().unwrap();
        let x_new = op(x_u);
        (self.k.clone(), x_new)
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Ast
{
    Block(Vec<Ast>),
    Call(Box<Ast>, LinkedList<Kxpr>, SrcLoc),
    Cons(Box<Ast>, Box<Ast>),
    ConstructData(DataType, Box<Ast>),
    ConstBool(bool),
    ConstHashtag(Lstr),
    ConstInt(i64),
    ConstStr(Lstr),
    ConstVoid,
    DefData(DataType, Box<Ast>, LinkedList<Kxpr>, SrcLoc),
    DefFunc(
        FuncClass,
        Box<Ast>,
        LinkedList<Kxpr>,
        Box<Ast>,
        Box<Ast>,
        SrcLoc,
    ),
    // dereference another expression
    Deref(Box<Ast>),
    DotAccess(Box<Ast>, Lstr),
    IfExpr(IfType, Box<Ast>, Box<IfCase>, SrcLoc),
    Import(Box<Ast>, SrcLoc),
    Let(LetType, Box<Ast>, Box<Ast>, SrcLoc),
    List(LinkedList<Ast>),
    Localid(Lstr, SrcLoc),
    Lri(Vec<Lstr>, Option<LinkedList<Ast>>, SrcLoc),
    Map(LinkedList<Kxpr>),
    Return(Box<Ast>, SrcLoc),
    RustBlock,
    StrExpr(Vec<Ast>, SrcLoc),
    Tuple(LinkedList<Kxpr>),
    TypeAnon,
    TypeBool,
    TypeFailure,
    TypeFunc(Vec<Kxpr>, SrcLoc),
    TypeInt,
    TypeHashtag,
    TypeStr,
    TypeVar(Lstr, SrcLoc),
    TypeVoid,
    Wildcard,
    /*
    LessThan3(Box<Ast>, Box<Ast>, Box<Ast>),
    */
}

impl Ast
{
    pub fn binaryop(callname: Vec<Lstr>, a: Ast, b: Ast, loc: SrcLoc) -> Ast
    {
        let name_lri = Ast::Lri(callname, None, loc.clone());
        let mut args = LinkedList::new();
        args.push_back(Kxpr::new_x(a));
        args.push_back(Kxpr::new_x(b));
        Ast::Call(Box::new(name_lri), args, loc)
    }

    pub fn from_lri(l: Lri, loc: &SrcLoc) -> Ast
    {
        if l.local_only() {
            return Ast::Localid(l.local_ref().clone(), *loc);
        }
        let mods = if l.has_modules() {
            vec![l.mod_ref().unwrap().clone(), l.local_ref().clone()]
        } else {
            vec![l.local_ref().clone()]
        };
        let new_params: Option<LinkedList<Ast>> = l.params.map(|params| {
            params.into_iter().map(|p| Ast::from_type(p, loc)).collect()
        });
        Ast::Lri(mods, new_params, *loc)
    }

    pub fn from_type(t: Type, loc: &SrcLoc) -> Ast
    {
        match t {
            Type::UserDef(lri) => Ast::from_lri(lri, loc),
            Type::Bool => Ast::TypeBool,
            Type::Int => Ast::TypeInt,
            Type::Str => Ast::TypeStr,
            Type::Hashtag => Ast::TypeHashtag,
            Type::Void => Ast::TypeVoid,
            Type::Tuple(items) => {
                let new_items = items
                    .0
                    .into_iter()
                    .map(|it| {
                        let newt = Ast::from_type(it.1, loc);
                        if it.0.is_some() {
                            Kxpr::new(it.0.unwrap(), newt)
                        } else {
                            Kxpr::new_x(newt)
                        }
                    }).collect();
                Ast::Tuple(new_items)
            }
            Type::Func(params, result) => {
                let kx_result = Kxpr::new_x(Ast::from_type(*result, loc));
                let mut new_params: Vec<Kxpr> = params
                    .into_iter()
                    .map(|it| {
                        let newt = Ast::from_type(it, loc);
                        Kxpr::new_x(newt)
                    }).collect();
                new_params.push(kx_result);
                Ast::TypeFunc(new_params, *loc)
            }
            Type::StrictList(inner) => {
                let mut items = LinkedList::new();
                items.push_back(Ast::from_type(*inner, loc));
                Ast::List(items)
            }
            Type::Var(name) => Ast::TypeVar(name, *loc),
            Type::AnonVar => Ast::TypeAnon,
            _ => {
                panic!("cannot convert from type to ast");
            }
        }
    }

    pub fn matchfunc_body(
        ids: &LinkedList<Kxpr>,
        cases: IfCase,
        loc: SrcLoc,
    ) -> Ast
    {
        let match_args = ids.iter().map(|idx| {
            match idx.k_ref() {
                None => {
                    panic!("cannot match function with unnamed parameter: {:?}"
                        , ids);
                }
                Some(id) => {
                    Kxpr::new_x(Ast::Localid(id.clone(), loc))
                }
            }
        }).collect();
        let test = Ast::Tuple(match_args);
        Ast::IfExpr(IfType::Match, Box::new(test), Box::new(cases), loc)
    }

    pub fn localid_str(&self) -> &Lstr
    {
        match self {
            &Ast::Localid(ref name, _) => name,
            _ => {
                panic!("not a localid: {:?}", self);
            }
        }
    }

    pub fn loc(&self) -> &SrcLoc
    {
        match self {
            &Ast::Localid(_, ref loc) => loc,
            &Ast::Lri(_, _, ref loc) => loc,
            _ => {
                panic!("cannot find SrcLoc for: {:?}", self);
            }
        }
    }

    pub fn inner_vec(&self) -> &Vec<Ast>
    {
        match self {
            &Ast::Block(ref items) => items,
            _ => {
                panic!("does not have inner vec: {:?}", self);
            }
        }
    }
}

impl<'a> From<&'a Ast> for String
{
    fn from(a: &'a Ast) -> String
    {
        match a {
            &Ast::Localid(ref ls, _) => String::from(ls),
            &Ast::Lri(ref items, ref types, _) => {
                format!("{:?}<{:?}>", items, types)
            }
            _ => {
                panic!("cannot convert to string: {:?}", a);
            }
        }
    }
}

impl<'a> From<&'a Ast> for Lstr
{
    fn from(a: &'a Ast) -> Lstr
    {
        match a {
            &Ast::Localid(ref ls, _) => ls.clone(),
            &Ast::Lri(ref items, ref types, _) => {
                if items.len() == 1 && types.is_none() {
                    items.first().unwrap().clone()
                } else {
                    let new_str = format!("{:?}<{:?}>", items, types);
                    Lstr::from(new_str)
                }
            }
            _ => {
                panic!("cannot convert to string: {:?}", a);
            }
        }
    }
}

impl<'a> From<&'a Ast> for Lri
{
    fn from(a: &'a Ast) -> Lri
    {
        match a {
            &Ast::Localid(ref id, _) => Lri::new(id.clone()),
            &Ast::Lri(ref names, ref types, _) => {
                let param_array = types.as_ref().map(|param_list| {
                    param_list.iter().map(|p| Type::from(p)).collect()
                });
                let modname = if names.len() == 1 {
                    None
                } else {
                    Some(names.first().unwrap().clone())
                };
                Lri::full(modname, names.last().unwrap().clone(), param_array)
            }
            _ => {
                panic!("cannot convert Ast to Lri: {:?}", a);
            }
        }
    }
}

impl<'a> From<&'a Ast> for Type
{
    fn from(a: &'a Ast) -> Type
    {
        match a {
            &Ast::TypeAnon => Type::AnonVar,
            &Ast::TypeInt => Type::Int,
            &Ast::TypeBool => Type::Bool,
            &Ast::TypeFailure => Type::Failure,
            &Ast::TypeHashtag => Type::Hashtag,
            &Ast::TypeStr => Type::Str,
            &Ast::TypeVar(ref v, _) => Type::Var(v.clone()),
            &Ast::TypeVoid => Type::Void,
            &Ast::TypeFunc(ref parts, _) => {
                let mut ppp: Vec<Type> = parts
                    .iter()
                    .map(|p| Type::from(p.x_ref().unwrap()))
                    .collect();
                let result = ppp.pop().unwrap();
                Type::Func(ppp, Box::new(result))
            }
            &Ast::List(ref items) => {
                match items.len() {
                    1 => {
                        let inner_item = items.front().unwrap();
                        let inner_type = Type::from(inner_item);
                        Type::StrictList(Box::new(inner_type))
                    }
                    0 => {
                        panic!("dunno what to do with an empty type list");
                    }
                    _ => {
                        panic!("list types can have only 1 type");
                    }
                }
            }
            &Ast::Tuple(ref items) => {
                let pp_items = items
                    .iter()
                    .map(|i| {
                        let new_k = i.k_ref().map(|kr| kr.clone());
                        (new_k, Type::from(i.x_ref().unwrap()))
                    }).collect();
                Type::Tuple(pp_items)
            }
            &Ast::Localid(ref id, _) => Type::UserDef(Lri::new(id.clone())),
            &Ast::Lri(_, _, _) => Type::UserDef(Lri::from(a)),
            _ => {
                panic!("cannot convert Ast to Type: {:?}", a);
            }
        }
    }
}


pub fn parse(toks: Vec<Token>) -> Ast
{
    let e = Err(0);
    let mut p = Parser::new(e);
    for t in toks {
        p.parse(t);
    }
    p.parse(Token::EOI);
    p.into_extra().unwrap()
}


#[cfg(test)]
mod tests
{
    use leema::ast::{self, Ast, Kxpr, Type};
    use leema::lex::lex;
    use leema::lri::Lri;
    use leema::lstr::Lstr;
    use leema::val::SrcLoc;

    use std::collections::LinkedList;


    fn test_lri(item: &'static str, line: i16, col: i8) -> Ast
    {
        Ast::Lri(
            vec![Lstr::from(item)].into_iter().collect(),
            None,
            SrcLoc::new(line, col),
        )
    }

    fn test_localid(id: &'static str, line: i16, col: i8) -> Ast
    {
        Ast::Localid(Lstr::from(id), SrcLoc::new(line, col))
    }

    fn test_mod_lri(
        m: &'static str,
        item: &'static str,
        line: i16,
        col: i8,
    ) -> Ast
    {
        Ast::Lri(
            vec![Lstr::from(m), Lstr::from(item)].into_iter().collect(),
            None,
            SrcLoc::new(line, col),
        )
    }

    #[test]
    fn test_from_lri_local_only()
    {
        let lri = Lri::new(Lstr::from("Tacos"));
        let ast = Ast::from_lri(lri.clone(), &SrcLoc::default());
        assert_eq!(lri, Lri::from(&ast));
    }

    #[test]
    fn test_from_lri_with_modules()
    {
        let i = Lri::with_modules(Lstr::from("burritos"), Lstr::from("Tacos"));
        let ast = Ast::from_lri(i.clone(), &SrcLoc::default());
        assert_eq!(i, Lri::from(&ast));
    }

    #[test]
    fn test_from_lri_with_params()
    {
        let inner_lri = Lri::full(
            Some(Lstr::from("option")),
            Lstr::from("T"),
            Some(vec![Type::Var(Lstr::from("U"))]),
        );
        let params = vec![
            Type::Int,
            Type::Var(Lstr::from("A")),
            Type::UserDef(inner_lri),
        ];
        let i = Lri::full(None, Lstr::from("Tacos"), Some(params));
        let ast = Ast::from_lri(i.clone(), &SrcLoc::default());
        assert_eq!(i, Lri::from(&ast));
    }

    #[test]
    fn test_from_lri_full()
    {
        let params = vec![Type::Var(Lstr::from("A"))];
        let i = Lri::full(
            Some(Lstr::from("burritos")),
            Lstr::from("Tacos"),
            Some(params),
        );
        let ast = Ast::from_lri(i.clone(), &SrcLoc::default());
        assert_eq!(i, Lri::from(&ast));
    }

    #[test]
    fn test_ast_parse_plus()
    {
        let input = "5 + 3\n";
        let lexed = lex(input);
        let root = ast::parse(lexed);

        let expected = Ast::Block(vec![Ast::Call(
            Box::new(test_mod_lri("prefab", "int_add", 1, 2)),
            vec![Kxpr::new_x(Ast::ConstInt(5)), Kxpr::new_x(Ast::ConstInt(3))]
                .into_iter()
                .collect(),
            SrcLoc::new(1, 2),
        )]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_ast_parse_strlit()
    {
        let input = "\"taco\"\n";
        let root = ast::parse(lex(input));

        let expected = Ast::Block(vec![Ast::ConstStr(Lstr::from("taco"))]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_ast_parse_string_id()
    {
        let input = "\"$var\"\n";
        let root = ast::parse(lex(input));

        let expected = Ast::Block(vec![Ast::StrExpr(
            vec![Ast::Localid(Lstr::from("var"), SrcLoc::new(1, 3))],
            SrcLoc::new(1, 1),
        )]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_ast_parse_string_list()
    {
        let input = "\"hello $name\n\"\n";
        let root = ast::parse(lex(input));

        let part1 = Ast::ConstStr(Lstr::from("hello "));
        let part2 = Ast::Localid(Lstr::from("name"), SrcLoc::new(1, 9));
        let part3 = Ast::ConstStr(Lstr::from("\n"));
        let expected = Ast::Block(vec![Ast::StrExpr(
            vec![part1, part2, part3],
            SrcLoc::new(1, 1),
        )]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_ast_parse_plus_twice()
    {
        let input = "5 + 3 + 2\n";
        let root = ast::parse(lex(input));

        let inner = Ast::Call(
            Box::new(test_mod_lri("prefab", "int_add", 1, 2)),
            vec![Kxpr::new_x(Ast::ConstInt(5)), Kxpr::new_x(Ast::ConstInt(3))]
                .into_iter()
                .collect(),
            SrcLoc::new(1, 2),
        );
        let outer = Ast::Call(
            Box::new(test_mod_lri("prefab", "int_add", 1, 4)),
            vec![Kxpr::new_x(inner), Kxpr::new_x(Ast::ConstInt(2))]
                .into_iter()
                .collect(),
            SrcLoc::new(1, 4),
        );

        let expected = Ast::Block(vec![outer]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_ast_parse_call_one_param()
    {
        let input = "inc(~4)\n";
        let root = ast::parse(lex(input));

        let neg4 = Ast::Call(
            Box::new(test_mod_lri("prefab", "int_negate", 1, 5)),
            vec![Kxpr::new_x(Ast::ConstInt(4))].into_iter().collect(),
            SrcLoc::new(1, 5),
        );
        let expected = Ast::Block(vec![Ast::Call(
            Box::new(Ast::Localid(Lstr::from("inc"), SrcLoc::new(1, 1))),
            vec![Kxpr::new_x(neg4)].into_iter().collect(),
            SrcLoc::new(1, 4),
        )]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_ast_parse_function_call()
    {
        let input = "foo(7, 2)\n";
        let root = ast::parse(lex(input));

        let xargs =
            vec![Kxpr::new_x(Ast::ConstInt(7)), Kxpr::new_x(Ast::ConstInt(2))]
                .into_iter()
                .collect();
        let expected = Ast::Block(vec![Ast::Call(
            Box::new(Ast::Localid(Lstr::from("foo"), SrcLoc::new(1, 1))),
            xargs,
            SrcLoc::new(1, 4),
        )]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_ast_parse_tuple()
    {
        let input = "(3, \"taco\", true)\n";
        let root = ast::parse(lex(input));

        let xtup = Ast::Tuple(
            vec![
                Kxpr::new_x(Ast::ConstInt(3)),
                Kxpr::new_x(Ast::ConstStr(Lstr::from("taco"))),
                Kxpr::new_x(Ast::ConstBool(true)),
            ].into_iter()
            .collect(),
        );
        let expected = Ast::Block(vec![xtup]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_ast_parse_list_empty()
    {
        let input = "[]\n";
        let root = ast::parse(lex(input));

        let xlist = Ast::List(vec![].into_iter().collect());
        let expected = Ast::Block(vec![xlist]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_ast_parse_const_list()
    {
        let input = "[1, 2, x]\n";
        let root = ast::parse(lex(input));

        let xlist = Ast::List(
            vec![
                Ast::ConstInt(1),
                Ast::ConstInt(2),
                Ast::Localid(Lstr::from("x"), SrcLoc::new(1, 6)),
            ].into_iter()
            .collect(),
        );
        let expected = Ast::Block(vec![xlist]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_ast_parse_list_cons()
    {
        let input = "1;2;x\n";
        let root = ast::parse(lex(input));

        let inner = Ast::Cons(
            Box::new(Ast::ConstInt(2)),
            Box::new(Ast::Localid(Lstr::from("x"), SrcLoc::new(1, 5))),
        );
        let outer = Ast::Cons(Box::new(Ast::ConstInt(1)), Box::new(inner));

        let expected = Ast::Block(vec![outer]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_call_function_plus_comma()
    {
        let input = "
    func main() ->
        foo(x+1, 40)
    --
    ";
        let root = ast::parse(lex(input));
        if let Ast::Block(items) = root {
            assert_eq!(1, items.len());
        } else {
            panic!("func def is not a block");
        }
    }

    #[test]
    fn test_parse_empty_tuple()
    {
        let input = "()";
        let root = ast::parse(lex(input));

        let xtuple = Ast::Tuple(LinkedList::new());
        let expected = Ast::Block(vec![xtuple]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_parse_one_tuple()
    {
        let input = "(5)";
        let root = ast::parse(lex(input));

        let tuple_items =
            vec![Kxpr::new_x(Ast::ConstInt(5))].into_iter().collect();
        let expected = Ast::Block(vec![Ast::Tuple(tuple_items)]);

        assert_eq!(expected, root);
    }

    #[test]
    fn test_parse_match_empty_list()
    {
        let input = "
    func is_empty(l)
    |([]) -> true
    |(_) -> false
    --
    ";
        ast::parse(lex(input));

        // didn't crash
        assert_eq!(2, 2);
    }

    #[test]
    fn test_call_function_comma_plus()
    {
        let input = "
    func main() ->
        foo(40, x+1)
    --
    ";
        ast::parse(lex(input));

        // didn't crash
        assert_eq!(3, 3);
    }

    #[test]
    fn test_parse_multiple_param_func()
    {
        let input = "
    func doubles(x, x2) ->
        x + x == x2
    --

    func main() ->
        doubles(5, 10)
    --
    ";
        ast::parse(lex(input));
    }

    #[test]
    fn test_ast_parse_if()
    {
        let input = "
    if x ->
        y
    else ->
        z
    --
    ";
        ast::parse(lex(input));

        // didn't crash
        assert_eq!(1, 1);
    }

    #[test]
    fn test_ast_parse_if_no_else()
    {
        let input = "if x -> y --";
        let root = ast::parse(lex(input));

        let blocka = ast::IfCase::new(
            Ast::Localid(Lstr::from("x"), SrcLoc::new(1, 3)),
            Ast::Block(vec![Ast::Localid(Lstr::from("y"), SrcLoc::new(1, 6))]),
            None,
            SrcLoc::new(1, 1),
        );

        let expected = Ast::Block(vec![Ast::IfExpr(
            ast::IfType::If,
            Box::new(Ast::ConstVoid),
            Box::new(blocka),
            SrcLoc::new(1, 1),
        )]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_ast_parse_macro()
    {
        let input = "
    macro mand(a, b) ->
        if
        |a -> b
        |else -> false
        --
    --
    ";
        let root = ast::parse(lex(input));

        if let Ast::Block(lines) = root {
            let f = lines.first().unwrap();
            if let &Ast::DefFunc(
                ast::FuncClass::Macro,
                ref name,
                ref args,
                _,
                _,
                _,
            ) = f
            {
                assert_eq!("mand", Lstr::from(&**name).str());
                assert_eq!(2, args.len());
            } else {
                panic!("mand is not a macro");
            }
        } else {
            panic!("mand does not have a block");
        }
    }

    #[test]
    fn test_parse_call_function_call_result()
    {
        let input = "(foo(5))(6)";
        let root = ast::parse(lex(input));

        let foo_call = Ast::Tuple(
            vec![Kxpr::new_x(Ast::Call(
                Box::new(test_localid("foo", 1, 2)),
                vec![Kxpr::new_x(Ast::ConstInt(5))].into_iter().collect(),
                SrcLoc::new(1, 5),
            ))].into_iter()
            .collect(),
        );

        let p_call = Ast::Call(
            Box::new(foo_call),
            vec![Kxpr::new_x(Ast::ConstInt(6))].into_iter().collect(),
            SrcLoc::new(1, 9),
        );

        let expected = Ast::Block(vec![p_call]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_parse_enum_variants()
    {
        let input = "
        enum Animal
        |Dog
        |Cat(Int)
        |Mouse($A)
        |Giraffe
            .height: Int
            .weight: $A
        --
    ";
        let root = ast::parse(lex(input));

        if let Ast::Block(lines) = root {
            let first = lines.first().unwrap();
            if let &Ast::DefData(ast::DataType::Enum, ref name, ref vars, _) =
                first
            {
                assert_eq!(
                    Ast::Localid(Lstr::from("Animal"), SrcLoc::new(2, 5)),
                    **name
                );
                assert_eq!(4, vars.len());
            } else {
                panic!("enum is not an enum: {:?}", first);
            }
        } else {
            panic!("enum is not a block: {:?}", root);
        }
    }

    #[test]
    fn test_parse_defstruple_tuple()
    {
        let input = "
    struct Taco(Int, Str)
    ";
        let root = ast::parse(lex(input));

        let def = Ast::DefData(
            ast::DataType::Struple,
            Box::new(test_localid("Taco", 2, 7)),
            vec![Kxpr::new_x(Ast::TypeInt), Kxpr::new_x(Ast::TypeStr)]
                .into_iter()
                .collect(),
            SrcLoc::new(2, 1),
        );
        assert_eq!(&def, root.inner_vec().get(0).unwrap());
    }

    #[test]
    fn test_parse_defstruple_keyed_params()
    {
        let input = "
    struct Taco(number: Int, style: Str)
    ";
        let root = ast::parse(lex(input));

        let def = Ast::DefData(
            ast::DataType::Struple,
            Box::new(test_localid("Taco", 2, 7)),
            vec![
                Kxpr::new(Lstr::from("number"), Ast::TypeInt),
                Kxpr::new(Lstr::from("style"), Ast::TypeStr),
            ].into_iter()
            .collect(),
            SrcLoc::new(2, 1),
        );
        assert_eq!(&def, root.inner_vec().get(0).unwrap());
    }

    #[test]
    fn test_parse_defstruple_mixed_keys()
    {
        let input = "
    struct Taco(Int, style: Str)
    ";
        let root = ast::parse(lex(input));

        let def = Ast::DefData(
            ast::DataType::Struple,
            Box::new(test_localid("Taco", 2, 7)),
            vec![
                Kxpr::new_x(Ast::TypeInt),
                Kxpr::new(Lstr::from("style"), Ast::TypeStr),
            ].into_iter()
            .collect(),
            SrcLoc::new(2, 1),
        );
        assert_eq!(&def, root.inner_vec().get(0).unwrap());
    }

    #[test]
    fn test_parse_defstruple_block()
    {
        let input = "
    struct Taco
    .number: Int
    .style: Str
    --
    ";
        let root = ast::parse(lex(input));

        let def = Ast::DefData(
            ast::DataType::Struple,
            Box::new(test_localid("Taco", 2, 7)),
            vec![
                Kxpr::new(Lstr::from("number"), Ast::TypeInt),
                Kxpr::new(Lstr::from("style"), Ast::TypeStr),
            ].into_iter()
            .collect(),
            SrcLoc::new(2, 1),
        );
        assert_eq!(&def, root.inner_vec().get(0).unwrap());
    }

    #[test]
    fn test_parse_match_list()
    {
        let input = "
    match x
    |(h;t) -> h
    |(_) -> false
    --
    ";
        let root = ast::parse(lex(input));

        let match_line = if let &Ast::Block(ref items) = &root {
            items.first().unwrap()
        } else {
            panic!("match is not a block");
        };

        if let &Ast::IfExpr(ast::IfType::Match, ref ifx, ref cases, _) =
            match_line
        {
            assert_eq!(Ast::Localid(Lstr::from("x"), SrcLoc::new(2, 6)), **ifx);
            assert!(cases.else_case.is_some());
        } else {
            panic!("match line not an if statement: {:?}", match_line);
        }
    }

    #[test]
    fn test_parse_constructor_call()
    {
        let input = "Taco(1, 2)";
        let root = ast::parse(lex(input));

        let call = Ast::Call(
            Box::new(test_localid("Taco", 1, 1)),
            vec![Kxpr::new_x(Ast::ConstInt(1)), Kxpr::new_x(Ast::ConstInt(2))]
                .into_iter()
                .collect(),
            SrcLoc::new(1, 5),
        );
        let expected = Ast::Block(vec![call]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_parse_strlit_field_access()
    {
        let input = "\"hello ${dog.name}\"";
        let root = ast::parse(lex(input));

        let strx = Ast::StrExpr(
            vec![
                Ast::ConstStr(Lstr::from("hello ")),
                Ast::DotAccess(
                    Box::new(test_localid("dog", 1, 8)),
                    Lstr::from("name"),
                ),
            ],
            SrcLoc::new(1, 1),
        );
        let expected = Ast::Block(vec![strx]);
        assert_eq!(expected, root);
    }

    #[test]
    fn test_parse_let_plus_negation()
    {
        let input = "
    let x := 4 + 8
    ~x
    ";
        let root = ast::parse(lex(input));

        if let Ast::Block(lines) = root {
            assert_eq!(2, lines.len());
        } else {
            panic!("root is not a block");
        }
    }

    #[test]
    fn test_parse_let_plus_tuple()
    {
        let input = "
    let x := 4 + y
    (x, z)
    ";
        let root = ast::parse(lex(input));

        if let Ast::Block(items) = root {
            assert_eq!(2, items.len());
        } else {
            panic!("root is not a block");
        }
    }

    #[test]
    fn test_parse_function_hashtag_tuple()
    {
        let input = "
    func foo(input: [(Int, #)]): [#] ->
        [#tacos, #burritos]
    --
    ";
        ast::parse(lex(input));

        assert!(true); // didn't panic!
    }

    #[test]
    fn test_parse_def_type_param()
    {
        let input = "
    enum Foo[A, B]
    |Bar(A)
    |Baz(B)
    --
    ";
        ast::parse(lex(input));

        assert!(true); // didn't panic!
    }

    #[test]
    fn test_parse_nested_type_param()
    {
        let input = "
    func foo(bar: Map[Int, opt::Option[V]]): opt::Option[V] ->
        option::None
    --
    ";
        ast::parse(lex(input));

        assert!(true); // didn't panic!
    }

    #[test]
    fn test_parse_function_type_param()
    {
        let input = "
    func call_func(i: Int, f: Int => Str => Int): Str => Int ->
        f(i)
    --
    ";
        ast::parse(lex(input));

        assert!(true); // didn't panic!
    }

}
