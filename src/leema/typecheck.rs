
use leema::ixpr::{Ixpr, Source};
use leema::infer::{Inferator};
use leema::lstr::{Lstr};
use leema::module::{ModKey};
use leema::program;
use leema::val::{Val, Type, TypeResult, TypeErr};
use leema::log;

use std::collections::{HashMap, LinkedList};
use std::io::{Write};
use std::rc::{Rc};

#[derive(Debug)]
pub enum CallOp
{
    LocalCall(Rc<String>),
    ExternalCall(Rc<String>, Rc<String>),
}

#[derive(Debug)]
pub struct CallFrame<'a>
{
    modname: &'a str,
    fname: &'a str,
    T: Inferator<'a>,
    pub calls: LinkedList<CallOp>,
}

impl<'a> CallFrame<'a>
{
    pub fn new(modname: &'a str, fname: &'a str) -> CallFrame<'a>
    {
        CallFrame{
            modname: modname,
            fname: fname,
            T: Inferator::new(fname),
            calls: LinkedList::new(),
        }
    }

    pub fn push_call(&mut self, call: CallOp)
    {
        self.calls.push_back(call);
    }

    pub fn pop_call(&mut self) -> Option<CallOp>
    {
        self.calls.pop_front()
    }

    pub fn collect_calls<'b>(&mut self, ix: &'b Ixpr)
    {
        match ix.src {
            Source::Call(ref callx, ref args) => {
                if let Source::Tuple(ref argsix) = args.src {
                    self.collect_calls_vec(argsix);
                    self.collect_callexpr(callx);
                }
            }
            Source::Block(ref expressions, ref fails, _is_root) => {
                self.collect_calls_vec(expressions);
                for case in fails.values() {
                    self.collect_calls(case);
                }
            }
            Source::Let(ref lhs, ref rhs, ref failed) => {
                self.collect_calls(rhs);
                // would be better to pass the iterator directly instead
                // of creating a new vector, but I don't know how to do
                // that right now and I'm only at this cafe for so long today
                let fails_only = failed.iter().map(|f| {
                    f.1.clone()
                }).collect();
                self.collect_calls_vec(&fails_only);
            }
            Source::StrMash(ref items) => {
                self.collect_calls_vec(items);
            }
            Source::IfExpr(ref cond, ref truth, ref lies) => {
                self.collect_calls(cond);
                self.collect_calls(truth);
                if lies.is_some() {
                    self.collect_calls(lies.as_ref().unwrap());
                }
            }
            Source::List(ref items) => {
                for i in items {
                    self.collect_calls(i);
                }
            }
            Source::Tuple(ref items) => {
                for i in items {
                    self.collect_calls(i);
                }
            }
            Source::ConstVal(ref val) => {
                // nothing to do. constants aren't calls.
            }
            Source::Id(ref _id, _) => {
                // nothing to do. not calls.
            }
            Source::MatchExpr(ref sample, ref cases) => {
                self.collect_calls(sample);
                self.collect_calls(cases);
            }
            Source::MatchCase(_, ref truth, ref lies) => {
                self.collect_calls(truth);
                self.collect_calls(lies);
            }
            Source::MatchFailure(ref x, ref cases) => {
                self.collect_calls(cases);
            }
            Source::Return(ref result) => {
                self.collect_calls(result);
            }
            Source::Func(ref _args, ref body) => {
                self.collect_calls(body);
            }
            Source::Cons(ref head, ref tail) => {
                self.collect_calls(head);
                self.collect_calls(tail);
            }
            Source::FieldAccess(ref base, _) => {
                self.collect_calls(base);
            }
            // nothing to do for these, not calls.
            Source::RustBlock => {}
            Source::Constructor(_, _) => {}
            Source::EnumConstructor(_, _, _) => {}
            Source::ModuleAccess(_, _) => {}
            Source::PropagateFailure(_, _) => {}
        }
    }

    pub fn collect_callexpr<'b>(&mut self, callx: &'b Ixpr)
    {
        match callx.src {
            Source::Id(ref callname, _) => {
                self.push_call(CallOp::LocalCall(callname.clone()));
            }
            Source::ModuleAccess(ref modname, ref callname) => {
                self.push_call(
                    CallOp::ExternalCall(modname.clone(), callname.clone()));
            }
            Source::ConstVal(ref val) => {
                match val {
                    &Val::Str(ref name) => {
                        self.push_call(CallOp::LocalCall(name.clone()));
                    }
                    &Val::Tuple(ref modfunc) if modfunc.len() == 2 => {
                        self.push_call(CallOp::ExternalCall(
                            modfunc.get(0).unwrap().to_str(),
                            modfunc.get(1).unwrap().to_str(),
                        ));
                    }
                    _ => {
                        panic!("Const val is not a call: {:?}", val);
                    }
                }
            }
            _ => {
                panic!("Unsupported call type: {:?}", callx);
            }
        }
    }

    pub fn collect_calls_vec<'b>(&mut self, xvec: &'b Vec<Ixpr>)
    {
        for x in xvec {
            self.collect_calls(x);
        }
    }
}

#[derive(Copy)]
#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
pub enum Depth
{
    Phase0,
    Inter,
    One,
    Full,
}

impl Depth
{
    pub fn one_deeper(&self) -> bool
    {
        match self {
            &Depth::Inter => false,
            &Depth::One => true,
            &Depth::Full => true,
            &Depth::Phase0 => {
                panic!("cannot check one_deeper for Depth::Phase0")
            }
        }
    }

    pub fn next(&self) -> Depth
    {
        match self {
            &Depth::One => Depth::Inter,
            &Depth::Full => Depth::Full,
            &Depth::Phase0 => {
                panic!("cannot get next for Depth::Phase0")
            }
            &Depth::Inter => {
                panic!("cannot get next for Depth::Inter")
            }
        }
    }
}

/**
 * Typemod
 *
 * Function scoped type checking
 * Type from preface
 * Type from first compilation
 * Type from one deep compilations
 * Type from full depth compilation
 */
#[derive(Debug)]
pub struct Typemod
{
    pub modname: Lstr,
    phase0: HashMap<Lstr, Type>,
    inter: HashMap<Lstr, Type>,
    depth_1: HashMap<Lstr, Type>,
    depth_full: HashMap<Lstr, Type>,
}

impl Typemod
{
    pub fn new(modname: Lstr) -> Typemod
    {
        Typemod{
            modname: modname,
            phase0: HashMap::new(),
            inter: HashMap::new(),
            depth_1: HashMap::new(),
            depth_full: HashMap::new(),
        }
    }

    pub fn import_phase0(&mut self, valtypes: &HashMap<String, Type>)
    {
        for (name, typ) in valtypes.iter() {
            self.phase0.insert(Lstr::from(name.clone()), typ.clone());
        }
    }

    pub fn name(&self) -> &str
    {
        self.modname.str()
    }

    pub fn set_type(&mut self, fname: Lstr, d: Depth, typ: Type)
    {
        match d {
            Depth::Phase0 => self.phase0.insert(fname, typ),
            Depth::Inter => self.inter.insert(fname, typ),
            Depth::One => self.depth_1.insert(fname, typ),
            Depth::Full => self.depth_full.insert(fname, typ),
        };
    }

    pub fn function_type(&self, fname: &Lstr) -> Option<&Type>
    {
        self.function_depth_type(fname, Depth::Full)
        .or_else(|| {
            self.function_depth_type(fname, Depth::One)
        })
        .or_else(|| {
            self.function_depth_type(fname, Depth::Inter)
        })
        .or_else(|| {
            self.function_depth_type(fname, Depth::Phase0)
        })
    }

    pub fn function_depth_type(&self, fname: &Lstr, d: Depth) -> Option<&Type>
    {
        match d {
            Depth::Phase0 => self.phase0.get(fname),
            Depth::Inter => self.inter.get(fname),
            Depth::One => self.depth_1.get(fname),
            Depth::Full => self.depth_full.get(fname),
        }
    }
}

#[derive(Debug)]
pub struct Typescope<'a, 'b>
{
    pub fname: &'b str,
    inter: &'a Typemod,
    imports: &'a HashMap<String, &'a Typemod>,
    T: Inferator<'b>,
}

impl<'a, 'b> Typescope<'a, 'b>
{
    pub fn new(inter: &'a Typemod, func: &'b str
            , imps: &'a HashMap<String, &'a Typemod>
            ) -> Typescope<'a, 'b>
    {
        Typescope
        {
            fname: func,
            inter: inter,
            imports: imps,
            T: Inferator::new(func),
        }
    }

    pub fn typecheck_matchcase(&mut self, valtype: &Type, case: &Ixpr
        ) -> TypeResult
    {
        match &case.src {
            &Source::MatchCase(ref patt, ref truth, ref lies) => {
                self.T.push_block(HashMap::new());
                self.T.match_pattern(patt, valtype, case.line);
                let ttype = typecheck_expr(self, truth).unwrap();
                self.T.pop_block();
                let ftype = self.typecheck_matchcase(valtype, lies).unwrap();

                self.T.merge_types(&ttype, &ftype)
            }
            &Source::ConstVal(Val::Void) => Ok(Type::Unknown),
            _ => {
                typecheck_expr(self, case)
            }
        }
    }

    pub fn typecheck_call_func(&mut self, src: &Source) -> TypeResult
    {
        match src {
            &Source::Tuple(ref items) => {
                if items.len() != 2 {
                    panic!("call tuples should have 2 items: {:?}", items);
                }
                let ref modname = items[0];
                let ref funcname = items[1];
                Ok(Type::Void)
            }
            &Source::ConstVal(ref fval) => {
                match fval {
                    &Val::Tuple(ref items) => {
                        let ref modname = items[0];
                        let ref funcname = items[1];
                        Ok(self.functype(modname.str(), funcname.str()))
                    }
                    &Val::Str(ref strname) => {
                        Ok(Type::Void)
                    }
                    &Val::FuncRef(_, _, ref typ) => {
                        Ok(typ.clone())
                    }
                    _ => {
                        panic!("what val is in typecheck_call? {:?}", fval);
                    }
                }
            }
            _ => {
                panic!("whatever is that in typecheck_call? {:?}", src);
            }
        }
    }

    pub fn functype(&self, modname: &str, funcname: &str) -> Type
    {
        let funclstr = Lstr::from(String::from(funcname));
        if modname == self.inter.name() {
            self.inter.function_type(&funclstr)
            .unwrap()
            .clone()
        } else {
            let m =
                self.imports.get(modname)
                .expect(&format!("cannot find module {} in {:?}"
                    , modname, self.imports));
            m.function_type(&funclstr)
                .expect(&format!("cannot find function {}::{} in {:?}"
                    , modname, funcname, m))
                .clone()
        }
    }
}

pub fn typecheck_expr(scope: &mut Typescope, ix: &Ixpr) -> TypeResult
{
    match &ix.src {
        &Source::Call(ref func, ref args) => {
            let tfunc = scope.typecheck_call_func(&func.src).unwrap();
            let mut targs = vec![];
            if let Source::Tuple(ref argstup) = args.src {
                for a in argstup {
                    targs.push(typecheck_expr(scope, a).unwrap());
                }
            } else {
                println!("args are not a tuple");
            }
            let mut targs_ref = vec![];
            for ta in targs.iter() {
                targs_ref.push(ta);
            }
            scope.T.make_call_type(&tfunc, &targs_ref)
        }
        &Source::ConstVal(ref cv) => {
            Ok(ix.typ.clone())
        }
        &Source::Let(ref lhs, ref rhs, _) => {
            let rhs_type = typecheck_expr(scope, rhs).unwrap();
            scope.T.match_pattern(lhs, &rhs_type, ix.line);
            Ok(Type::Void)
        }
        &Source::Block(ref elems, ref fails, _is_root) => {
            let mut last_type = Ok(Type::Void);
            for e in elems {
                last_type = typecheck_expr(scope, e);
            }
            for f in fails.values() {
                typecheck_expr(scope, f);
            }
            last_type
        }
        &Source::Id(_, _) => {
            Ok(ix.typ.clone())
        }
        &Source::List(ref items) => {
            for i in items {
                typecheck_expr(scope, i);
            }
            Ok(ix.typ.clone())
        }
        &Source::Tuple(ref items) => {
            let mut tuptyp = vec![];
            for i in items {
                tuptyp.push(typecheck_expr(scope, i).unwrap());
            }
            Ok(Type::Tuple(tuptyp))
        }
        &Source::IfExpr(ref cond, ref truth, ref lies) => {
            let cond_t = typecheck_expr(scope, cond).unwrap();
            scope.T.merge_types(&cond_t, &Type::Bool);

            let truth_result = typecheck_expr(scope, truth);
            match lies {
                &None => truth_result,
                &Some(ref some_lies) => {
                    let lies_result = typecheck_expr(scope, some_lies);
                    let truth_t = truth_result.unwrap();
                    let lies_t = lies_result.unwrap();
                    scope.T.merge_types(&truth_t, &lies_t)
                }
            }
        }
        &Source::StrMash(ref items) => {
            for i in items {
                typecheck_expr(scope, i);
                // TODO: check that this supports the stringification interface
                // let it = typecheck_expr(scope, i);
                // scope.T.match_types(&it, &Type::Str);
            }
            Ok(Type::Str)
        }
        &Source::Func(ref _args, ref body) => {
            // typecheck_expr(scope, body)
            Err(TypeErr::Error(Rc::new(
                format!("unexpected func in typecheck: {:?}", body)
                )))
        }
        &Source::MatchExpr(ref subject, ref cases) => {
            typecheck_expr(scope, subject)
                .and_then(|subject_type| {
                    scope.typecheck_matchcase(&subject_type, cases)
                })
        }
        &Source::MatchCase(_, _, _) => {
            panic!("typecheck matchcase in a specific function: {:?}", ix);
        }
        _ => {
            panic!("Could not typecheck_expr({:?})", ix);
        }
    }
}

pub fn typecheck_function(scope: &mut Typescope, ix: &Ixpr) -> TypeResult
{
    vout!("typecheck function({:?}: {:?})", scope.fname, ix.typ);
    vout!("typescope: {:?}\n", scope);
    match (&ix.src, &ix.typ) {
        (&Source::Func(ref arg_names, ref body)
                , &Type::Func(ref arg_types, ref declared_result_type)) =>
        {
            for (an, at) in arg_names.iter().zip(arg_types.iter()) {
                scope.T.bind_vartype(an, at, ix.line);
            }
            println!("f({:?}) =>\n{:?}", arg_names, body);
            let result_type = typecheck_expr(scope, &*body).unwrap();
            println!("type is: {}", result_type);
            println!("vars:");
            for var in scope.T.vars() {
                let typ = scope.T.vartype(var);
                println!("\t{}: {}", var, typ.unwrap());
            }
            let final_args = arg_types.iter().map(|at| {
                scope.T.inferred_type(at).clone()
            }).collect();
            scope.T.merge_types(&result_type, declared_result_type)
                .map(|final_type| {
                    Type::Func(final_args, Box::new(final_type))
                })
        }
        (&Source::RustBlock, _) => {
            Ok(ix.typ.clone())
        }
        _ => {
            panic!("Cannot typecheck_function a not function: {:?}", ix);
        }
    }
    /*
    scope.add_parameters(fix)
    */
}


#[cfg(test)]
mod tests {
    use leema::program;
    use leema::loader::{Interloader};
    use leema::log;

    use std::io::{Write};


#[test]
#[should_panic]
fn test_pattern_type_inferred_mismatch()
{
    let input = String::from("

    ## foo should take [#] and return a #
    func foo(inputs)
    |([]) -> #empty
    |(#whatever;more) -> #whatever
    |(_;more) -> foo(more)
    --

    func main() ->
        foo([5, 3, 4])
    --
    ");

    let mut loader = Interloader::new("tacos.lma");
    loader.set_mod_txt("tacos", input);
    let mut prog = program::Lib::new(loader);
    prog.deep_typecheck("tacos", "main");
}

}

/*
mod -> prog
imps -> libs

load_func(mn, fn) {
    mod = prog.load_module(prog, mn)
    import_mods(mod)
    if !mod_sourced(mn)
        source_mod(mn)
    if !mod_loaded(mn)
        load_mod(mn)
    mod = get_mod(mn)
    if !mod.imports_loaded
        load_imports(mod.imports)
    src_func = get_func(mod, fn)
    pfunc = preprocess(src_func)
    tfunc = type_func(pfunc)
    if !func_preprocessed(mod, fn)
        src_func
        preprocess(src_func)
    if !func_processed(mod, fn)
}

for f in mod.funcs {
    typecheck_func(f)
}

typecheck_call(m, f, args) {
    tfd = typecheck_func(m, f)
    typecheck(args, tfd.args)
}

typecheck_func(m, f) {
    fd = getf(m, f)
    for e in fd {
        typecheck_expr(e)
    }
}

typecheck_expr(e) {
    if e == call {
        typecheck_call(call.module, call.func, call.args)
    }
}
*/

/*
pub fn module(prog: &mut program::Lib, inter: &Interloader, modname: &str)
{
    vout!("typecheck.module({})\n", modname);
    let imported_mods = {
        let m = prog.get_mod_mut(modname);
        println!("loaded module:\n{:?}\n", m);

        let mut imods = Vec::with_capacity(m.src.imports.len());
        for i in &m.src.imports {
            println!("import({})", i);
            let ikey = inter.mod_name_to_key(&i);
            let mut im = inter.init_module(ikey);
            im.load();
            imods.push(im);
        }
        m.imports_loaded = true;
        imods
    };

    for i in imported_mods {
        println!("imported({:?})", i);
        prog.add_mod(i);
    }

    let funcs: Vec<String> = {
        prog.get_mod(modname).src.funcs.keys().map(|im| {
            im.clone()
        }).collect()
    };
    // let mut ts = new TypeSpace();
    // ts.add_imports(&m.src.imports);

    for fname in funcs {
        func(prog, inter, &fname);
    }
}
*/

/*
thing that reads files

thing that holds imported files, ntyped functions

view of visible naively, unferred typed stuff

thing that holds complete type info

view of visible type-complete stuff

library of typed code


pub fn function(prog: &mut program::Lib, inter: &Interloader,
        modname: &str, fname: &str) -> Ixpr
{
    let mut m = prog.load_module(modname);
    let mut tf = m.typed_func(fname);
    if tf.is_none() {
        sf = m.src_func(fname);
        tf = preprocess(sf, prog);
    }
    match m.typed_func(fname) {
        None => j
    }
    let tfunc = prog.typed_func(
    let fsrc = prog.get_func(modname, fname);
}
*/
