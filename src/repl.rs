use leema::val::{Env};
use leema::frame::{Frame, Application, Parent};
use leema::lex::{lex};
use leema::ast::{Ast};
use leema::val::{Val};
use leema::compile::{StaticSpace};
use leema::code::{Code, Op, make_ops};

use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{stdin, stdout, Write};


fn prompt() {
    write!(stdout(), "> ").ok();
    stdout().flush().ok();
}

fn read_cmd() -> String {
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).ok();
    buffer
}

/*
fn apply_macro(mdef: Sxpr, input: List) -> Sxpr {
    match mdef {
        Sxpr::MacroDef(_, ref args, ref code) => {
            if input_items.len() != args.len() {
                panic!("Wrong number of args");
            }
            let mut i = 0;
            let mut argval: HashMap<&String, Rc<Ast>> = HashMap::new();
            for arg in args {
                argval.insert(arg, input_items[i].clone());
                i = i + 1;
            }
            println!("argval: {:?}", argval);
            // do nothing
            // Ast::Bool(true)
            let mut new_code = Vec::with_capacity(code.len());
            for c in code {
                new_code.push(Ast::replace_id(c, &argval));
            }
            Sxpr::Block(List::Nil)
        }
        _ => {
            panic!("That was not a macro");
        }
    }
}
*/


pub fn push_eval(app: &Mutex<Application>, i: isize, function: Code, e: Env)
{
    println!("exec {:?}", function);

    let frm = Frame::new_root(e);
//println!("repl.push_eval: app.lock().unwrap()");
    let mut _app = app.lock().unwrap();
    _app.push_new_frame(frm);
    _app.add_code(ckey, function);
}

pub fn wait_eval(app: Arc<Mutex<Application>>) -> Val
{
//println!("repl.wait_eval: app.lock().unwrap()");
    let mut result = Val::Void;
    // this is crap. set this w/ a channel or something.
    thread::yield_now();
    let mut _app = app.lock().unwrap();
//println!("repl.wait_eval: app.pop_old_frame().unwrap()");
    _app.take_result()
}

pub fn reploop(app: Arc<Mutex<Application>>, mut ss: StaticSpace)
{
    let mut i = 1;
    let mut e = Env::new();
    loop {
        prompt();
        let input = read_cmd();

        let tokens = lex(input);
        println!("tokens = {:?}\n", tokens);

        let ast_root = Ast::parse(tokens);
        println!("ast = {:?}", ast_root);
        let rootx = Ast::root(ast_root);

        // static compilation
        let inter = ss.compile(rootx);
        println!("inter> {:?}", inter);

        let ops = make_ops(&inter);
        println!("ops>");
        Op::print_list(&ops);
        println!("---\n");

        let code = Code::Leema(Arc::new(ops));
        push_eval(&*app, i, code, e);
        //let result = Worker::eval(cmd_code);
        // println!("= {:?}", result);

        let mut result = wait_eval(app.clone());
        // TODO: but really, extract the env out of the execution
        e = Env::new();
        println!("= {:?}", result);
        i += 1;
    }
}
