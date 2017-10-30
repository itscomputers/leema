%include {
use leema::ast::{TokenLoc, TokenData};
use leema::val::{Val, SxprType, Type};
use leema::list;
use leema::log;
use leema::sxpr;
use std::io::{stderr, Write};
use std::rc::{Rc};
}

%start_symbol {program}
%derive_token {Debug,Clone,PartialEq}
%wildcard ANY.
%extra_argument { Result<Val, i32> }

%type COLON { TokenLoc }
%type COMMA { TokenLoc }
%type DBLCOLON { TokenLoc }
%type DOUBLEDASH { TokenLoc }
%type ELSE { TokenLoc }
%type HASHTAG { TokenData<String> }
%type ID { TokenData<String> }
%type INT { i64 }
%type Let { TokenLoc }
%type LPAREN { TokenLoc }
%type RPAREN { TokenLoc }
%type PLUS { TokenLoc }
%type SEMICOLON { TokenLoc }
%type SLASH { TokenLoc }
%type SquareL { TokenLoc }
%type SquareR { TokenLoc }
%type StrLit { String }
%type TYPE_VAR { TokenData<String> }

%type program { Val }
%type stmts { Val }

%type control_stmt { Val }
%type result_stmt { Val }
%type block { Val }
%type func_stmt { Val }
%type dfunc_args { Val }
%type dfunc_one { Val }
%type dfunc_many { Val }
%type failed_stmt { Val }
%type failed_stmts { Val }
%type macro_stmt { Val }
%type macro_args { Val }
%type call_expr { Val }
%type call_args { Val }
%type call_arg { Val }
%type if_stmt { Val }
%type else_if { Val }
%type if_expr { Val }
%type if_case { Val }
%type match_case { Val }
%type defstruct { Val }
%type defstruct_fields { Val }
%type defstruct_field { Val }
%type let_stmt { Val }
%type fail_stmt { Val }
%type term { Val }
%type control_expr { Val }
%type sub_expr { Val }
%type typex { Type }
%type opt_typex { Type }
%type mod_prefix { Val }
%type tuple_types { Val }
%type id_type { Val }

%type list { Val }
%type list_items { Val }
%type tuple { Val }
%type tuple_args { Val }
%type strexpr { Val }
%type strlist { Val }
%type strlist_term { Val }


%nonassoc ASSIGN BLOCKARROW RETURN.
%left OR XOR.
%left AND.
%right ConcatNewline NOT.
%nonassoc EQ NEQ GT GTEQ.
%left LT LTEQ.
%right SEMICOLON.
%left PLUS MINUS.
%left TIMES SLASH MOD.
%right DOLLAR.
%left DOT.
%left LPAREN RPAREN.

%parse_accept {
	//println!("parse accepted");
}

%syntax_error {
    match token {
        &Token::EOI => {
	        panic!("Unexpected end of file. Maybe add newline?");
        }
        _ => {
	        println!("syntax error at token {:?}\n", token);
	        panic!("Syntax error. wtf? @ <line,column>?");
        }
    }
}

%parse_failure {
	println!("parse failure");
	panic!("Parse failed.");
}

program(A) ::= stmts(B). {
	// ignore A, it doesn't really go anywhere for program
	A = Val::Void;
	// we're done, so put B in extra
	self.extra = Ok(B);
}

stmts(A) ::= . {
	A = list::empty();
}
stmts(A) ::= result_stmt(B). {
	A = B;
}
stmts(A) ::= control_stmt(C) stmts(B). {
	A = list::cons(C, B);
}


control_stmt(A) ::= defstruct(B). { A = B; }
control_stmt(A) ::= IMPORT ID(B). {
    A = sxpr::new_import(Val::id(B.data));
}
control_stmt(A) ::= IMPORT mod_prefix(B). {
    A = sxpr::new_import(B);
}
control_stmt(A) ::= let_stmt(B). { A = B; }
control_stmt(A) ::= fail_stmt(B). { A = B; }
control_stmt(A) ::= func_stmt(B). { A = B; }
control_stmt(A) ::= macro_stmt(B). { A = B; }
control_stmt(A) ::= if_stmt(B). { A = B; }
control_stmt(A) ::= control_expr(B). { A = B; }
result_stmt(A) ::= sub_expr(B). { A = B; }
result_stmt(A) ::= RETURN sub_expr(B). {
    A = sxpr::new(SxprType::Return, B);
}


/** Data Structures */
defstruct(A) ::= STRUCT typex(B) defstruct_fields(C) DOUBLEDASH. {
    A = sxpr::def_struct(Val::Type(B), C);
}
defstruct_fields(A) ::= defstruct_field(B) defstruct_fields(C). {
	A = list::cons(B, C);
}
defstruct_fields(A) ::= . {
	A = list::empty();
}
defstruct_field(A) ::= DOT ID(B) COLON typex(C). {
	A = Val::typed_id(&B.data, C);
}


fail_stmt(A) ::= FAIL(B) LPAREN HASHTAG(C) COMMA sub_expr(D) RPAREN. {
vout!("found fail_stmt {:?}\n", C);
	A = sxpr::new(SxprType::Fail,
        list::cons(Val::hashtag(C.data),
        list::cons(D,
        Val::Nil,
        ))
    );
}

failed_stmt(A) ::= FAILED ID(B) match_case(C) DOUBLEDASH. {
	A = sxpr::new(SxprType::MatchFailed,
        list::cons(Val::id(B.data),
        list::cons(C,
        Val::Nil
        ))
    );
}

let_stmt(A) ::= Let ID(B) ASSIGN control_expr(C). {
    let letx =
        list::cons(Val::id(B.data),
        list::cons(C,
        Val::Nil
        ));
    A = sxpr::new(SxprType::Let, letx);
}
let_stmt(A) ::= Fork ID(B) ASSIGN control_expr(C). {
	let bind = list::cons(Val::new_str(B.data), list::singleton(C));
	A = sxpr::new(SxprType::Fork, bind);
}
let_stmt ::= Let ID(B) EQ1 expr. {
    panic!("Found let {} = <expr>\nit should be let {} := <expr>\n",
        B.data, B.data);
}

block(A) ::= BLOCKARROW stmts(B) failed_stmts(C). {
	A = sexpr::new(SexprType::BlockExpr, list::from2(B, C));
}

/* rust func declaration */
func_stmt(A) ::= Func ID(B) PARENCALL dfunc_args(D) RPAREN COLON typex(E)
    RUSTBLOCK.
{
	let id = Val::id(B.data);
	let typ = Val::Type(E);
	A = sxpr::defunc(id, D, typ, Val::RustBlock, Val::Void)
}

/* func one case, no matching */
func_stmt(A) ::= Func ID(B) PARENCALL dfunc_args(D) RPAREN opt_typex(E)
    block(C) DOUBLEDASH.
{
	let id = Val::id(B.data);
	let typ = Val::Type(E);
	A = sxpr::defunc(id, D, typ, C, F)
}
/* func w/ pattern matching */
func_stmt(A) ::= Func ID(B) PARENCALL dfunc_args(C) RPAREN opt_typex(D)
    match_case(E) DOUBLEDASH.
{
	let id = Val::id(B.data);
	let typ = Val::Type(D);
    let body = sxpr::match_expr(Val::CallParams, E);
	A = sxpr::defunc(id, C, typ, body, F)
}

dfunc_args(A) ::= . {
	A = list::empty();
}
dfunc_args(A) ::= ID(B) opt_typex(C). {
	A = list::singleton(Val::typed_id(&B.data, C));
}
dfunc_args(A) ::= ID(B) opt_typex(C) COMMA dfunc_args(D). {
	A = list::cons(Val::typed_id(&B.data, C), D);
}



opt_typex(A) ::= . {
	A = Type::AnonVar;
}
opt_typex(A) ::= COLON typex(B). {
	A = B;
}

typex(A) ::= TYPE_INT. {
	A = Type::Int;
}
typex(A) ::= TYPE_STR. {
	A = Type::Str;
}
typex(A) ::= TYPE_HASHTAG. {
	A = Type::Hashtag;
}
typex(A) ::= TYPE_BOOL. {
	A = Type::Bool;
}
typex(A) ::= TYPE_VOID. {
	A = Type::Void;
}
typex(A) ::= TYPE_VAR(B). {
	A = Type::Var(Rc::new(B.data));
}
typex(A) ::= ID(B). {
	A = Type::Id(Rc::new(B.data));
}
typex(A) ::= SquareL typex(B) SquareR. {
	A = Type::StrictList(Box::new(B));
}
typex(A) ::= LPAREN tuple_types(B) RPAREN. {
    A = Type::Tuple(list::map_to_vec(B, |v| {
        v.to_type()
    }));
}
tuple_types(A) ::= . {
    A = Val::Nil;
}
tuple_types(A) ::= typex(B). {
    A = list::singleton(Val::Type(B));
}
tuple_types(A) ::= typex(B) COMMA tuple_types(C). {
    A = list::cons(Val::Type(B), C);
}

failed_stmts(A) ::= . {
    A = list::empty();
}
failed_stmts(A) ::= failed_stmt(B) failed_stmts(C). {
    A = list::cons(B, C);
}


/* defining a macro */
macro_stmt(A) ::= MACRO ID(B) PARENCALL macro_args(D) RPAREN block(C) DOUBLEDASH. {
    vout!("found macro {:?}\n", B);
    A = sxpr::new(SxprType::DefMacro,
        list::cons(Val::id(B.data),
        list::cons(D,
        list::cons(C,
        Val::Nil
    ))));
}

macro_args(A) ::= . {
    A = Val::Nil;
}
macro_args(A) ::= ID(B). {
    A = list::singleton(Val::id(B.data));
}
macro_args(A) ::= ID(B) COMMA macro_args(C). {
    A = list::cons(Val::id(B.data), C);
}

control_expr(A) ::= call_expr(B). { A = B; }
control_expr(A) ::= if_expr(B). { A = B; }
control_expr(A) ::= match_expr(B). { A = B; }

sub_expr(A) ::= term(B). { A = B; }

/* if statements can go 3 different ways
* might be that one of these should be the only one, but I'm not sure
* now and not ready to commit, so leaving all 3 in.
*
* part of the reason for indecision is concern about incorrectly
* expecting a full block to be inside of a single expr if/else block
* when it is not
*
* if only style (requires stmt block to avoid if-block bugs):
*     if x ->
*         y
*     --
*
* if/else style:
*     if x -- y
*     else if y -- z
*     else z -- whatever
*     --
*
* case expr style:
*     if
*     |x -- y
*     |y -- z
*     |z -- whatever
*     --
*/
if_stmt(A) ::= IF sub_expr(B) block(C) DOUBLEDASH. {
    /* if-only style */
    A = sxpr::ifx(B, C, Val::Void);
}
if_stmt(A) ::= IF sub_expr(B) block(C) else_if(D) DOUBLEDASH. {
    /* if-else style */
    A = sxpr::ifx(B, C, D);
}
else_if(A) ::= ELSE IF sub_expr(B) block(C) else_if(D). {
    A = sxpr::ifx(B, C, D);
}
else_if(A) ::= ELSE IF sub_expr(B) block(C). {
    A = sxpr::ifx(B, C, Val::Void);
}
else_if(A) ::= ELSE block(B). {
    A = B;
}


/* regular function call */
call_expr(A) ::= term(B) PARENCALL RPAREN. {
	A = sxpr::call(B, Val::Nil);
}
call_expr(A) ::= term(B) PARENCALL call_args(C) RPAREN. {
	A = sxpr::call(B, C);
}

call_arg(A) ::= sub_expr(B). {
    A = B;
}
call_arg(A) ::= ID(B) COLON sub_expr(C). {
    let name = Val::id(B.data);
    A = sxpr::named_param(name, C);
}
call_args(A) ::= call_arg(B). {
    A = list::singleton(B);
}
call_args(A) ::= call_arg(B) COMMA. {
    A = list::singleton(B);
}
call_args(A) ::= call_arg(B) COMMA call_args(C). {
    A = list::cons(B, C);
}

sub_expr(A) ::= term(B) DOLLAR term(C). {
	/* A = Val::binaryop(B, C, D); */
	A = Val::Void;
}

/* IF expression */
if_expr(A) ::= IF if_case(B) DOUBLEDASH. {
    /* case-expr style */
    A = B;
}
if_case(A) ::= PIPE sub_expr(B) block(C). {
    A = sxpr::ifx(B, C, Val::Void);
}
if_case(A) ::= PIPE sub_expr(B) block(C) if_case(D). {
    A = sxpr::ifx(B, C, D);
}
if_case(A) ::= PIPE ELSE block(B). {
    A = B;
}

/* match expression */
match_expr(A) ::= MATCH sub_expr(B) match_case(C) DOUBLEDASH. {
    A = sxpr::match_expr(B, C);
}
match_case(A) ::= PIPE LPAREN call_args(B) RPAREN block(C) match_case(D). {
    let patt = Val::tuple_from_list(B);
    A = list::from3(patt, C, D);
}
match_case(A) ::= PIPE LPAREN call_args(B) RPAREN block(C). {
    let patt = Val::tuple_from_list(B);
    A = list::from2(patt, C);
}


expr(A) ::= list(B). { A = B; }
/* tuple
 * (4 + 4, 6 - 7)
 */
sub_expr(A) ::= NOT sub_expr(B). {
	A = sxpr::call(Val::id("bool_not".to_string()), list::singleton(B));
}
sub_expr(A) ::= sub_expr(B) ConcatNewline. {
	let newline = Val::new_str("\n".to_string());
	let args = list::cons(B, list::singleton(newline));
	A = sxpr::new(SxprType::StrExpr, args)
}
/* arithmetic */
sub_expr(A) ::= NEGATE term(B). {
	A = sxpr::call(Val::id("int_negate".to_string()), list::singleton(B));
}
sub_expr(A) ::= sub_expr(B) PLUS sub_expr(C). {
	A = sxpr::binaryop("int_add".to_string(), B, C);
}
sub_expr(A) ::= sub_expr(B) MINUS sub_expr(C). {
	A = sxpr::binaryop("int_sub".to_string(), B, C);
}
sub_expr(A) ::= sub_expr(B) TIMES sub_expr(C). {
	A = sxpr::binaryop("int_mult".to_string(), B, C);
}
sub_expr(A) ::= sub_expr(B) SLASH sub_expr(C). {
	A = sxpr::binaryop("int_div".to_string(), B, C);
}
sub_expr(A) ::= sub_expr(B) MOD sub_expr(C). {
	A = sxpr::binaryop("int_mod".to_string(), B, C);
}
sub_expr(A) ::= sub_expr(B) SEMICOLON sub_expr(C). {
	A = list::cons(B, C);
}
sub_expr(A) ::= sub_expr(B) AND sub_expr(C). {
	A = sxpr::binaryop("boolean_and".to_string(), B, C);
}
sub_expr(A) ::= sub_expr(B) OR sub_expr(C). {
	A = sxpr::binaryop("boolean_or".to_string(), B, C);
}
sub_expr(A) ::= sub_expr(B) XOR sub_expr(C). {
	A = sxpr::binaryop("boolean_xor".to_string(),B, C);
}

/* comparisons */
sub_expr(A) ::= sub_expr(B) LT sub_expr(C). {
	A = sxpr::binaryop("less_than".to_string(), B, C);
}
sub_expr(A) ::= sub_expr(B) LTEQ sub_expr(C). {
	A = sxpr::binaryop("less_than_equal".to_string(), B, C);
}
sub_expr(A) ::= sub_expr(B) GT sub_expr(C). {
	A = sxpr::binaryop("greater_than".to_string(), B, C);
}
sub_expr(A) ::= sub_expr(B) GTEQ sub_expr(C). {
	A = sxpr::binaryop("greater_than_equal".to_string(), B, C);
}
sub_expr(A) ::= sub_expr(B) EQ(P) sub_expr(C). {
	A = sxpr::binaryop("equal".to_string(), B, C);
}
sub_expr(A) ::= sub_expr(B) NEQ(P) sub_expr(C). {
	let eq = sxpr::binaryop("equal".to_string(), B, C);
	A = sxpr::call(Val::id("bool_not".to_string()), list::singleton(eq));
}
/*
expr(A) ::= expr(B) LT expr(C) LT expr(D). {
	let l1 = sxpr::binaryop("less_than".to_string(), B, C);
	let l2 = sxpr::binaryop("less_than".to_string(), C, D);
	A = sxpr::binaryop("and".to_string(), l1, l2);
}
expr(A) ::= expr(B) LT expr(C) LTEQ expr(D). {
	let l1 = sxpr::binaryop("less_than".to_string(), B, C);
	let l2 = sxpr::binaryop("less_than_equal".to_string(), C, D);
	A = sxpr::binaryop("and".to_string(), l1, l2);
}
expr(A) ::= expr(B) LTEQ expr(C) LT expr(D). {
	let l1 = sxpr::binaryop("less_than_equal".to_string(), B, C);
	let l2 = sxpr::binaryop("less_than".to_string(), C, D);
	A = sxpr::binaryop("and".to_string(), l1, l2);
}
expr(A) ::= expr(B) LTEQ expr(C) LTEQ expr(D). {
	let l1 = sxpr::binaryop("less_than_equal".to_string(), B, C);
	let l2 = sxpr::binaryop("less_than_equal".to_string(), C, D);
	A = sxpr::binaryop("and".to_string(), l1, l2);
}
*/

term(A) ::= LPAREN sub_expr(C) RPAREN. {
	A = C;
}
term(A) ::= tuple(B). {
    A = B;
}
term(A) ::= ID(B). { A = Val::id(B.data); }
term(A) ::= mod_prefix(B). {
    A = B;
}
term(A) ::= VOID. {
	A = Val::Void;
}
term(A) ::= DollarQuestion. {
	A = Val::id("$".to_string());
}
term(A) ::= INT(B). {
	A = Val::Int(B);
}
term(A) ::= True. { A = Val::Bool(true); }
term(A) ::= False. { A = Val::Bool(false); }
term(A) ::= HASHTAG(B). {
	A = Val::Hashtag(Rc::new(B.data));
}
term(A) ::= strexpr(B). { A = B; }
term(A) ::= UNDERSCORE. { A = Val::Wildcard; }
term(A) ::= term(B) DOT ID(C). {
    A = Val::DotAccess(Box::new(B), Rc::new(C.data));
}

mod_prefix(A) ::= ID(B) DBLCOLON ID(C). {
    A = Val::ModPrefix(Rc::new(B.data), Rc::new(Val::id(C.data)));
}
mod_prefix(A) ::= ID(B) DBLCOLON mod_prefix(C). {
    A = Val::ModPrefix(Rc::new(B.data), Rc::new(C));
}

list(A) ::= SquareL SquareR. {
	A = list::empty();
}
list(A) ::= SquareL list_items(B) SquareR. {
	A = B;
}
list_items(A) ::= expr(B). {
	A = list::singleton(B);
}
list_items(A) ::= expr(B) COMMA list_items(C). {
	A = list::cons(B, C);
}

tuple(A) ::= LPAREN RPAREN. {
    A = Val::Tuple(vec![]);
}
tuple(A) ::= LPAREN tuple_args(B) RPAREN. {
	A = Val::tuple_from_list(B);
}
tuple_args(A) ::= expr(B) COMMA expr(C). {
	A = list::cons(B, list::singleton(C));
}
tuple_args(A) ::= expr(B) COMMA tuple_args(C). {
	A = list::cons(B, C);
}

strexpr(A) ::= StrOpen strlist(B) StrClose. {
	A = sxpr::strexpr(B);
    vout!("strexpr({:?})\n", A);
}
strlist(A) ::= . {
	A = Val::Nil;
}
strlist(A) ::= StrLit(B) strlist(C). {
	A = list::cons(Val::new_str(B), C);
}
strlist(A) ::= strlist_term(B) strlist(C). {
	A = list::cons(B, C);
}
strlist_term(A) ::= ID(B). {
    A = Val::id(B.data);
}
strlist_term(A) ::= strlist_term(B) DOT ID(C). {
    A = Val::DotAccess(Box::new(B), Rc::new(C.data));
}
