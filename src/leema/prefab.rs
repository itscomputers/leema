use leema::val::{Val, Type};
use leema::code::{Code};
use leema::frame::{Frame};
use leema::compile::{Compiler, StaticSpace};
use leema::list;
use leema::log;
use leema::ast;
use std::fs::File;
use std::io::{stderr, Read, Write};
use std::any::{Any};
use std::fmt::{self, Display, Debug};
use std::sync::{Arc, Mutex};


pub fn int_add(fs: &mut Frame)
{
    let ic;
    {
        let a = fs.get_param(0);
        let b = fs.get_param(1);
        match (a,b) {
            (&Val::Int(ia), &Val::Int(ib)) => {
                ic = ia + ib;
            }
            _ => {
                panic!("wtf is all that? {:?}", (a,b));
            }
        }
    }
    fs.parent.set_result(Val::Int(ic));
}

pub fn int_sub(fs: &mut Frame)
{
    let ic;
    {
        let a = fs.get_param(0);
        let b = fs.get_param(1);
        match (a,b) {
            (&Val::Int(ia), &Val::Int(ib)) => {
                ic = ia - ib;
            }
            _ => {
                panic!("wtf is all that? {:?}", (a,b));
            }
        }
    }
    fs.parent.set_result(Val::Int(ic));
}

pub fn int_mult(fs: &mut Frame)
{
    let ic;
    {
        let a = fs.get_param(0);
        let b = fs.get_param(1);
        match (a,b) {
            (&Val::Int(ia), &Val::Int(ib)) => {
                ic = ia * ib;
            }
            _ => {
                panic!("can't multiply that! {:?}", (a,b));
            }
        }
    }
    fs.parent.set_result(Val::Int(ic));
}

pub fn int_mod(fs: &mut Frame)
{
    let ic;
    {
        let a = fs.get_param(0);
        let b = fs.get_param(1);
        match (a,b) {
            (&Val::Int(ia), &Val::Int(ib)) => {
                ic = ia % ib;
            }
            _ => {
                panic!("can't mod that! {:?}", (a,b));
            }
        }
    }
    fs.parent.set_result(Val::Int(ic));
}

pub fn int_negate(fs: &mut Frame)
{
    let result;
    {
        let a = fs.get_param(0);
        match a {
            &Val::Int(a) => {
                result = -a;
            }
            _ => {
                panic!("can't negate a not int? {:?}", a);
            }
        }
    }
    fs.parent.set_result(Val::Int(result));
}

pub fn bool_not(fs: &mut Frame)
{
    println!("run bool_not!");
    let bnot;
    {
        let bval = fs.get_param(0);
        bnot = match bval {
            &Val::Bool(b) => !b,
            _ => {
                panic!("wtf is all that? {:?}", bval);
            }
        }
    }
    fs.parent.set_result(Val::Bool(bnot));
}

pub fn bool_xor(fs: &mut Frame)
{
    let result;
    {
        let va = fs.get_param(0);
        let vb = fs.get_param(1);
        match (va,vb) {
            (&Val::Bool(a), &Val::Bool(b)) => {
                result = a && !b || b && !a;
            }
            _ => {
                panic!("wtf is all that? {:?}", (va,vb));
            }
        }
    }
    fs.parent.set_result(Val::Bool(result));
}

pub fn list_cons(fs: &mut Frame)
{
    let head = fs.e.get_param(0);
    let tail = fs.e.get_param(1);
    fs.parent.set_result(list::cons(head.clone(), tail.clone()));
}

pub fn less_than(fs: &mut Frame)
{
    let result;
    {
        let va = fs.get_param(0);
        let vb = fs.get_param(1);
        result = va < vb;
    }
    fs.parent.set_result(Val::Bool(result));
}

pub fn less_than_equal(fs: &mut Frame)
{
    let va = fs.e.get_param(0);
    let vb = fs.e.get_param(1);
    fs.parent.set_result(Val::Bool(va <= vb));
}

pub fn equal(fs: &mut Frame)
{
    let result;
    {
        let va = fs.get_param(0);
        let vb = fs.get_param(1);
        result = va == vb;
    }
    fs.parent.set_result(Val::Bool(result));
}

pub fn greater_than(fs: &mut Frame)
{
    let result;
    {
        let va = fs.get_param(0);
        let vb = fs.get_param(1);
        result = va > vb;
    }
    fs.parent.set_result(Val::Bool(result));
}

pub fn greater_than_equal(fs: &mut Frame)
{
    let result;
    {
        let va = fs.get_param(0);
        let vb = fs.get_param(1);
        result = va >= vb;
    }
    fs.parent.set_result(Val::Bool(result));
}

pub fn get_type(fs: &mut Frame)
{
    let result;
    {
        let v = fs.get_param(0);
        result = v.get_type();
    }
    fs.parent.set_result(Val::Type(result));
}

pub fn cout(fs: &mut Frame)
{
    {
        let va = fs.get_param(0);
        print!("{}", va);
    }
    fs.parent.set_result(Val::Void);
}

pub fn cerr(fs: &mut Frame)
{
    {
        let va = fs.get_param(0);
        write!(stderr(), "{}", va);
    }
    fs.parent.set_result(Val::Void);
}


struct LeemaFile {
    f: Mutex<File>,
}

impl LeemaFile
{
    pub fn new(f: File) -> LeemaFile
    {
        LeemaFile{f: Mutex::new(f)}
    }
}

impl Display for LeemaFile
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "LeemaFile")
    }
}

impl Debug for LeemaFile
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "LeemaFile")
    }
}


pub fn file_read(fs: &mut Frame)
{
    let open_result = {
        let fnval = fs.get_param(0);
        match fnval {
            &Val::Str(ref fnstr) => {
                File::open(&**fnstr)
            }
            _ => {
                panic!("Can't open file with not string {:?}"
                    , fnval);
            }
        }
    };
    let openf = match open_result {
        Ok(f) => {
            Val::new_lib(
                LeemaFile::new(f),
                Type::Lib("File".to_string()),
            )
        }
        Err(_) => Val::failure(
            Val::hashtag("file_open_fail".to_string()),
            Val::new_str("Failed to open file".to_string()),
            fs.trace.failure_here(),
            )
    };
    fs.parent.set_result(openf);
}

pub fn stream_read_file(fs: &mut Frame)
{
    let mut input = "".to_string();
    {
        let mut streamval = fs.get_param_mut(0);
        let mut anyf = match streamval {
            &mut Val::Lib(ref lv) =>
            {
                &*lv.v as &Any
            }
            _ => {
                panic!("Can't read from not a File {:?}"
                    , streamval);
            }
        };
        let mut optf = Any::downcast_ref::<LeemaFile>(anyf);
        let mut myfref = optf.unwrap();
        let mut lockf = myfref.f.lock();
        let mut rawf = lockf.unwrap();
        let mut result = rawf.read_to_string(&mut input);
        //let result = myf.f.lock().unwrap().read_to_string(&mut input);
    }
println!("read from file: '{}'", input);
    fs.parent.set_result(Val::Str(Arc::new(input)));
}

fn define_macros(ss: &mut StaticSpace)
{
    vout!("defining prefab macros\n");
    let input = "
    macro boolean_and(a, b) ->
        case
        |a -> b
        |else -> false
        --
    --

    macro boolean_or(a, b) ->
        case
        |a -> true
        |else -> b
        --
    --
    ".to_string();

    let mut loader = ast::Loader::new();
    loader.set_file("prefab_macros".to_string(), input);
    let mut c = Compiler::new(ss, loader);
    c.compile_file("prefab_macros".to_string());
    vout!("prefab macros compiled\n");
}

pub fn define_prefab(ss: &mut StaticSpace)
{
    // set up static space
    ss.define_func(Arc::new("int_add".to_string()),
        Type::f(vec![Type::Int, Type::Int], Type::Int),
        Code::Rust(int_add),
        );
    ss.define_func(Arc::new("int_sub".to_string()),
        Type::f(vec![Type::Int, Type::Int], Type::Int),
        Code::Rust(int_sub),
        );
    ss.define_func(Arc::new("int_mult".to_string()),
        Type::f(vec![Type::Int, Type::Int], Type::Int),
        Code::Rust(int_mult),
        );
    ss.define_func(Arc::new("int_mod".to_string()),
        Type::f(vec![Type::Int, Type::Int], Type::Int),
        Code::Rust(int_mod),
        );
    ss.define_func(Arc::new("negate".to_string()),
        Type::f(vec![Type::Int], Type::Int),
        Code::Rust(int_negate),
        );
    ss.define_func(Arc::new("bool_not".to_string()),
        Type::f(vec![Type::Bool], Type::Bool),
        Code::Rust(bool_not),
        );
    ss.define_func(Arc::new("xor".to_string()),
        Type::f(vec![Type::Bool, Type::Bool], Type::Bool),
        Code::Rust(bool_xor),
        );
    let type_var_a = Type::Var(Arc::new("TypeA".to_string()));
    ss.define_func(Arc::new("list_cons".to_string()),
        Type::f(vec![
            type_var_a.clone(),
            Type::StrictList(Box::new(type_var_a.clone())),
        ], Type::StrictList(Box::new(type_var_a.clone()))),
        Code::Rust(list_cons),
        );
    ss.define_func(Arc::new("less_than".to_string()),
        Type::f(vec![Type::Int, Type::Int], Type::Bool),
        Code::Rust(less_than),
        );
    ss.define_func(Arc::new("less_than_equal".to_string()),
        Type::f(vec![Type::Int, Type::Int], Type::Bool),
        Code::Rust(less_than_equal),
        );
    ss.define_func(Arc::new("equal".to_string()),
        Type::f(vec![Type::Int, Type::Int], Type::Bool),
        Code::Rust(equal),
        );
    ss.define_func(Arc::new("greater_than".to_string()),
        Type::f(vec![Type::Int, Type::Int], Type::Bool),
        Code::Rust(greater_than),
        );
    ss.define_func(Arc::new("greater_than_equal".to_string()),
        Type::f(vec![Type::Int, Type::Int], Type::Bool),
        Code::Rust(greater_than_equal),
        );
    ss.define_func(Arc::new("type".to_string()),
        Type::f(vec![Type::Any], Type::Kind),
        Code::Rust(get_type),
        );
    ss.define_func(Arc::new("cout".to_string()),
        Type::f(vec![Type::Str], Type::Void),
        Code::Rust(cout),
        );
    ss.define_func(Arc::new("file_read".to_string()),
        Type::f(vec![Type::Str], Type::Lib("File".to_string())),
        Code::Rust(file_read),
        );
    ss.define_func(Arc::new("stream_read".to_string()),
        Type::f(vec![Type::Lib("File".to_string())], Type::Void),
        Code::Rust(stream_read_file),
        );

    define_macros(ss);
}

pub fn new_staticspace() -> StaticSpace
{
    let mut ss = StaticSpace::new();
    define_prefab(&mut ss);
    ss
}
