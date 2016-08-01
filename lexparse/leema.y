%include {
use leema::ast::{Ast, TokenLoc, TokenData};
use leema::val::{Val, SexprType, Type};
use leema::list;
use leema::log;
use leema::sexpr;
use std::sync::Arc;
use std::io::{stderr, Write};
}

%start_symbol {program}
%derive_token {Debug,PartialEq}
%wildcard ANY.
%extra_argument { Result<Ast, i32> }

%type COMMA { TokenLoc }
%type ELSE { TokenLoc }
%type HASHTAG { TokenData<String> }
%type ID { String }
%type INT { i64 }
%type PLUS { TokenLoc }
%type SLASH { TokenLoc }
%type StrLit { String }
%type TYPE_ID { TokenData<String> }

%type program { Ast }
%type stmts { Val }

%type stmt { Val }
%type match_block { Val }
%type block { Val }
%type func_stmt { Val }
%type dfunc_args { Val }
%type dfunc_one { Val }
%type dfunc_many { Val }
%type macro_stmt { Val }
%type macro_args { Val }
%type call_id { Val }
%type if_stmt { Val }
%type else_if { Val }
%type if_case { Val }
%type match_case { Val }
%type pexpr { Val }
%type ptuple { Val }
%type pargs { Val }
%type def_struct { Val }
%type struct_fields { Val }
%type struct_field { Val }
%type let_stmt { Val }
%type expr_stmt { Val }
%type fail_stmt { Val }
%type term { Val }
%type expr { Val }
%type typex { Type }
%type opt_typex { Type }
%type id_type { Val }
%type cases { Val }

%type list { Val }
%type list_items { Val }
%type tuple { Val }
%type tuple_args { Val }
%type strexpr { Val }
%type strlist { Val }


%nonassoc ASSIGN BLOCKARROW WHERE GIVEN.
%left OR XOR.
%left AND.
%right ConcatNewline NOT.
%nonassoc EQ NEQ GT GTEQ.
%left LT LTEQ.
%left PLUS MINUS.
%left TIMES SLASH MOD.
%nonassoc LPAREN RPAREN.

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

program(A) ::= FAILED WITH CONTEXTID. {
	A = Ast::Nothing;
}
program(A) ::= stmts(B). {
	if list::is_empty(&B) {
		panic!("null program");
	}
	// ignore A, it doesn't really go anywhere for program
	A = Ast::Nothing;
	// we're done, so put B in extra
	self.extra = Ok(Ast::ReplRoot(B));
}

stmts(A) ::= . {
	A = sexpr::new(SexprType::BlockExpr, list::empty());
}
stmts(A) ::= stmt(C) stmts(B). {
    verbose_out!("found new stmt: {:?}\n", C);
	A = list::cons(C, B);
}


stmt(A) ::= def_struct(B). { A = B; }
stmt(A) ::= let_stmt(B). { A = B; }
stmt(A) ::= expr_stmt(B). {
    A = B;
}
stmt(A) ::= fail_stmt(B). { A = B; }
stmt(A) ::= func_stmt(B). { A = B; }
stmt(A) ::= macro_stmt(B). { A = B; }
/* if_stmt */
stmt(A) ::= if_stmt(B). { A = B; }

/*
stmt(A) ::= FAILED(B) ID(C) match_block(D). {
	A = new FailureStmt(C->text, D);
	A->merge_span(*B, *D);
}
*/


/** Data Structures */
def_struct(A) ::= STRUCT typex(B) struct_fields(C) DOUBLEDASH. {
    A = sexpr::def_struct(Val::Type(B), C);
}
struct_fields(A) ::= struct_field(B) struct_fields(C). {
	A = list::cons(B, C);
}
struct_fields(A) ::= . {
	A = list::empty();
}
struct_field(A) ::= DOT ID(B) COLON typex(C). {
	A = Val::Tuple(vec![Val::id(B), Val::Type(C)]);
}


fail_stmt(A) ::= FAIL(B) HASHTAG(C) term(D). {
verbose_out!("found fail_stmt {:?}\n", C);
	/*
	A = Val::list(
		list::push(B,
		list::push(C,
		list::push(D,
		NULL))));
		*/
	A = Val::Failure;
}

let_stmt(A) ::= Let ID(B) EQ expr(C). {
	let letx =
        list::cons(Val::id(B),
        list::cons(C,
        Val::Nil
        ));
	A = sexpr::new(SexprType::Let, letx);
}
let_stmt(A) ::= Fork ID(B) EQ expr(C). {
	let bind = list::cons(Val::new_str(B), list::singleton(C));
	A = sexpr::new(SexprType::Fork, bind);
}

expr_stmt(A) ::= expr(B). {
	A = B;
}
expr_stmt(A) ::= DollarGT expr(B). { A = B; }

block(A) ::= BLOCKARROW stmts(B). {
	A = B;
}

/* func one case, no matching */
func_stmt(A) ::= Func ID(B) LPAREN dfunc_args(D) RPAREN opt_typex(E)
    block(C) DOUBLEDASH.
{
	let id = Val::id(B);
	let typ = Val::Type(E);
	A = sexpr::defunc(id, D, typ, C)
}
/* func w/ pattern matching */
func_stmt(A) ::= Func ID(B) LPAREN dfunc_args(C) RPAREN opt_typex(D)
    match_case(E) DOUBLEDASH.
{
	let id = Val::id(B);
	let typ = Val::Type(D);
    let body = sexpr::match_expr(Val::CallParams, E);
	A = sexpr::defunc(id, C, typ, body)
}

dfunc_args(A) ::= . {
	A = list::empty();
}
dfunc_args(A) ::= ID(B) opt_typex(C). {
	A = list::singleton(sexpr::id_with_type(B, C));
}
dfunc_args(A) ::= ID(B) opt_typex(C) COMMA dfunc_args(D). {
	A = list::cons(sexpr::id_with_type(B, C), D);
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
typex(A) ::= TYPE_BOOL. {
	A = Type::Bool;
}
typex(A) ::= TYPE_VOID. {
	A = Type::Void;
}
typex(A) ::= TYPE_ID(B). {
	A = Type::Id(Arc::new(B.data));
}


/* defining a macro */
macro_stmt(A) ::= MACRO ID(B) LPAREN macro_args(D) RPAREN block(C) DOUBLEDASH. {
    verbose_out!("found macro {:?}\n", B);
    A = sexpr::new(SexprType::DefMacro,
        list::cons(Val::id(B),
        list::cons(D,
        list::cons(C,
        Val::Nil
    ))));
}

macro_args(A) ::= . {
    A = Val::Nil;
}
macro_args(A) ::= ID(B). {
    A = list::singleton(Val::id(B));
}
macro_args(A) ::= ID(B) COMMA macro_args(C). {
    A = list::cons(Val::id(B), C);
}

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
if_stmt(A) ::= IF expr(B) block(C) DOUBLEDASH. {
    /* if-only style */
    A = sexpr::ifstmt(B, C, Val::Void);
}
if_stmt(A) ::= IF expr(B) block(C) else_if(D) DOUBLEDASH. {
    /* if-else style */
    A = sexpr::ifstmt(B, C, D);
}
if_stmt(A) ::= IF if_case(B) DOUBLEDASH. {
    /* case-expr style */
    A = B;
}
else_if(A) ::= ELSE IF expr(B) block(C) else_if(D). {
    A = sexpr::ifstmt(B, C, D);
}
else_if(A) ::= ELSE IF expr(B) block(C). {
    A = sexpr::ifstmt(B, C, Val::Void);
}
else_if(A) ::= ELSE block(B). {
    A = B;
}
if_case(A) ::= PIPE expr(B) block(C) if_case(D). {
    A = sexpr::ifstmt(B, C, D);
}
if_case(A) ::= PIPE ELSE block(B). {
    A = B;
}


/* regular function call */
expr(A) ::= expr(B) LPAREN RPAREN. {
	verbose_out!("zero param function call!");
	A = sexpr::call(B, vec![]);
}
expr(A) ::= expr(B) LPAREN expr(C) RPAREN. {
	verbose_out!("one param function call!");
	A = sexpr::call(B, vec![C]);
}
expr(A) ::= expr(B) LPAREN tuple_args(C) RPAREN. {
	verbose_out!("multi param function call!");
	A = sexpr::call(B, list::to_vec(C));
}
/* postfix function call, are we really doing this?
seems like this should be pretty achievable w/ `[] | empty?`
expr(A) ::= term(B) ID(C). {
	A = Sexpr::Nothing;
}*/
/* infix function call */
expr(A) ::= term(B) ID(C) term(D). {
	A = sexpr::binaryop(C, B, D);
}

expr(A) ::= term(B) DOLLAR term(C). {
	/* A = Val::binaryop(B, C, D); */
	A = Val::Void;
}
/* CASE expression */
expr(A) ::= CASE cases(B) DOUBLEDASH. {
    verbose_out!("parsed case expr\n");
	A = B;
}
cases(A) ::= PIPE expr(B) block(C) PIPE ELSE block(D). {
    verbose_out!("found cases base\n");
    A = sexpr::casex(B, C, D);
}
cases(A) ::= PIPE expr(B) block(C) cases(D). {
    verbose_out!("found extra case\n");
    A = sexpr::casex(B, C, D);
}

/* match expression */
expr(A) ::= MATCH expr(B) match_case(C) DOUBLEDASH. {
    verbose_out!("parsed match expr\n");
    A = sexpr::match_expr(B, C);
}
match_case(A) ::= PIPE pexpr(B) block(C) match_case(D). {
    verbose_out!("found cases base\n");
    A = sexpr::match_case(B, C, D);
}
match_case(A) ::= PIPE pexpr(B) block(C). {
    verbose_out!("parsed base match case\n");
    A = sexpr::match_case(B, C, Val::Void);
}

pexpr(A) ::= ptuple(B). { A = B; }
pexpr(A) ::= INT(B). { A = Val::Int(B); }
pexpr(A) ::= True. { A = Val::Bool(true); }
pexpr(A) ::= False. { A = Val::Bool(false); }
pexpr(A) ::= HASHTAG(B). { A = Val::Hashtag(Arc::new(B.data)); }
pexpr(A) ::= ID(B). { A = Val::id(B); }
pexpr(A) ::= UNDERSCORE. { A = Val::Wildcard; }
ptuple ::= LPAREN RPAREN. {
	panic!("an empty tuple is not a valid pattern");
}
ptuple(A) ::= LPAREN pexpr(B) RPAREN. {
	A = Val::Tuple(vec![B]);
}
ptuple(A) ::= LPAREN pargs(B) RPAREN. {
	A = Val::tuple_from_list(B);
}
pargs(A) ::= pexpr(B) COMMA pexpr(C). {
	A = list::cons(B,
        list::cons(C,
        Val::Nil
        ));
}
pargs(A) ::= pexpr(B) COMMA pargs(C). {
	A = list::cons(B, C);
}


expr(A) ::= list(B). { A = B; }
/* tuple
 * (4 + 4, 6 - 7)
 */
expr(A) ::= tuple(B). { A = B; }
expr(A) ::= NOT expr(B). {
	A = sexpr::call(Val::id("bool_not".to_string()), vec![B]);
}
expr(A) ::= expr(B) ConcatNewline. {
	let newline = Val::Str(Arc::new("\n".to_string()));
	let args = list::cons(B, list::singleton(newline));
	A = sexpr::new(SexprType::StrExpr, args)
}
/* arithmetic */
expr(A) ::= MINUS expr(B). {
	A = sexpr::call(Val::id("negate".to_string()), vec![B]);
}
expr(A) ::= expr(B) PLUS expr(C). {
	A = sexpr::binaryop("int_add".to_string(), B, C);
}
expr(A) ::= expr(B) MINUS expr(C). {
	A = sexpr::binaryop("int_sub".to_string(), B, C);
}
expr(A) ::= expr(B) TIMES expr(C). {
	A = sexpr::binaryop("int_mult".to_string(), B, C);
}
expr(A) ::= expr(B) SLASH expr(C). {
	A = sexpr::binaryop("int_div".to_string(), B, C);
}
expr(A) ::= expr(B) MOD expr(C). {
	A = sexpr::binaryop("int_mod".to_string(), B, C);
}
expr(A) ::= expr(B) AND expr(C). {
	A = sexpr::binaryop("and".to_string(), B, C);
}
expr(A) ::= expr(B) OR expr(C). {
	A = sexpr::binaryop("or".to_string(), B, C);
}
expr(A) ::= expr(B) XOR expr(C). {
	A = sexpr::binaryop("xor".to_string(),B, C);
}

/* comparisons */
expr(A) ::= expr(B) LT expr(C). {
	A = sexpr::binaryop("less_than".to_string(), B, C);
}
expr(A) ::= expr(B) LTEQ expr(C). {
	A = sexpr::binaryop("less_than_equal".to_string(), B, C);
}
expr(A) ::= expr(B) GT expr(C). {
	A = sexpr::binaryop("greater_than".to_string(), B, C);
}
expr(A) ::= expr(B) GTEQ expr(C). {
	A = sexpr::binaryop("greater_than_equal".to_string(), B, C);
}
expr(A) ::= expr(B) EQ(P) expr(C). {
	A = sexpr::binaryop("equal".to_string(), B, C);
}
expr(A) ::= expr(B) NEQ(P) expr(C). {
	let eq = sexpr::binaryop("equal".to_string(), B, C);
	A = sexpr::call(Val::id("bool_not".to_string()), vec![eq]);
}
/*
expr(A) ::= expr(B) LT expr(C) LT expr(D). {
	let l1 = sexpr::binaryop("less_than".to_string(), B, C);
	let l2 = sexpr::binaryop("less_than".to_string(), C, D);
	A = sexpr::binaryop("and".to_string(), l1, l2);
}
expr(A) ::= expr(B) LT expr(C) LTEQ expr(D). {
	let l1 = sexpr::binaryop("less_than".to_string(), B, C);
	let l2 = sexpr::binaryop("less_than_equal".to_string(), C, D);
	A = sexpr::binaryop("and".to_string(), l1, l2);
}
expr(A) ::= expr(B) LTEQ expr(C) LT expr(D). {
	let l1 = sexpr::binaryop("less_than_equal".to_string(), B, C);
	let l2 = sexpr::binaryop("less_than".to_string(), C, D);
	A = sexpr::binaryop("and".to_string(), l1, l2);
}
expr(A) ::= expr(B) LTEQ expr(C) LTEQ expr(D). {
	let l1 = sexpr::binaryop("less_than_equal".to_string(), B, C);
	let l2 = sexpr::binaryop("less_than_equal".to_string(), C, D);
	A = sexpr::binaryop("and".to_string(), l1, l2);
}
*/

expr(A) ::= term(B). { A = B; }


term(A) ::= LPAREN expr(C) RPAREN. {
	A = C;
}
term(A) ::= typex(B). {
    A = Val::Type(B);
}
term(A) ::= ID(B). { A = Val::id(B); }
/* term(A) ::= var_field(B). { A = Ast::Nothing; } */
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
	A = Val::Hashtag(Arc::new(B.data));
}
term(A) ::= strexpr(B). { A = B; }

	/*
var_field(A) ::= ID(B) DOT ID(C) var_field_depth(D). {
	A = Val::list(
		list::cons(B,
		list::cons(C,
		D->data.list)), LET_SUBSYMBOL);
}
var_field_depth(A) ::= . {
	// A = Val::empty_list(LET_SUBSYMBOL);
}
var_field_depth(A) ::= DOT ID(B) var_field_depth(C). {
	// A = Val::cons(B, C);
}
		*/

list(A) ::= SquareL list_items(B) SquareR. {
	A = B;
}
list_items(A) ::= . {
	A = list::empty();
}
list_items(A) ::= expr(B). {
	A = list::singleton(B);
}
list_items(A) ::= expr(B) COMMA list_items(C). {
	A = list::cons(B, C);
}

tuple(A) ::= LPAREN tuple_args(B) RPAREN. {
	A = Val::tuple_from_list(B);
}
tuple_args(A) ::= expr(B) COMMA expr(C). {
	verbose_out!("base tuple args!");
	A = list::cons(B, list::singleton(C));
}
tuple_args(A) ::= expr(B) COMMA tuple_args(C). {
	verbose_out!("additional tuple arg!");
	A = list::cons(B, C);
}

strexpr(A) ::= StrOpen strlist(B) StrClose. {
	A = sexpr::strexpr(B);
}
strlist(A) ::= . {
	A = Val::Nil;
}
strlist(A) ::= StrLit(B) strlist(C). {
	A = list::cons(Val::new_str(B), C);
}
strlist(A) ::= ID(B) strlist(C). {
	A = list::cons(Val::id(B), C);
}
