
use leema::log;
use leema::ast::{Ast};
use leema::lstr::{Lstr};
use leema::val::{Val, Type, SrcLoc, TypeResult, TypeErr};

use std::collections::{HashMap};
use std::collections::hash_map::Keys;
use std::io::{Write};
use std::rc::{Rc};


#[derive(Debug)]
pub struct VarData
{
    failure: Option<Ast>,
    assignment: Option<i16>,
    first_usage: Option<SrcLoc>,
    must_check_failure: bool,
}

impl VarData
{
    pub fn new(failures: Ast) -> VarData
    {
        VarData{
            failure: Some(failures),
            assignment: None,
            first_usage: None,
            must_check_failure: false,
        }
    }
}

impl Default for VarData
{
    fn default() -> VarData
    {
        VarData{
            failure: None,
            assignment: None,
            first_usage: None,
            must_check_failure: false,
        }
    }
}

#[derive(Debug)]
pub struct Blockscope
{
    vars: HashMap<String, VarData>,
    failing: bool,
}

impl Blockscope
{
    pub fn new(failures: HashMap<String, Ast>) -> Blockscope
    {
        let vars: HashMap<String, VarData> =
            failures.into_iter().map(|(v, fail)| {
                (v, VarData::new(fail))
            }).collect();
        Blockscope{
            vars: vars,
            failing: false,
        }
    }
}

#[derive(Debug)]
pub struct Inferator<'b>
{
    funcname: &'b str,
    T: HashMap<String, Type>,
    blocks: Vec<Blockscope>,
    inferences: HashMap<Rc<String>, Type>,
    module: Option<Rc<String>>,
}

impl<'b> Inferator<'b>
{
    pub fn new(funcname: &'b str) -> Inferator<'b>
    {
        Inferator{
            funcname: funcname,
            T: HashMap::new(),
            blocks: vec![Blockscope::new(HashMap::new())],
            inferences: HashMap::new(),
            module: None,
        }
    }

    pub fn vars(&self) -> Keys<String, Type>
    {
        self.T.keys()
    }

    pub fn vartype(&self, argn: &str) -> Option<Type>
    {
        match self.T.get(argn) {
            None => None,
            Some(&Type::AnonVar) => {
                panic!("Can't infer AnonVar");
            }
            Some(ref argt) => {
                Some(self.inferred_type(argt))
            }
        }
    }

    pub fn init_param(&mut self, argi: i16, argn: Option<&Lstr>
        , argt: Type, line: i16
        ) -> TypeResult
    {
        vout!("bind_vartype({}, #{} {:?}: {:?})\n"
            , self.funcname, argi, argn, argt);
        let b = self.blocks.last_mut().unwrap();
        if argn.is_some() {
            // just assign the var b/c it's a new param
            let mut vdata = VarData::default();
            vdata.assignment = Some(line);
            b.vars.insert(String::from(argn.unwrap()), vdata);
        }

        let realt = match argt {
            Type::Unknown => {
                let arg_typename = format!("T_param_{}", argi);
                Type::Var(Rc::new(arg_typename))
            }
            Type::AnonVar => {
                let arg_typename = format!("T_param_{}", argi);
                Type::Var(Rc::new(arg_typename))
            }
            a => a,
        };
        if argn.is_some() {
            let argn_u = argn.unwrap();
            if self.T.contains_key(argn_u.str()) {
                let oldargt = self.T.get(argn_u.str()).unwrap();
                return Inferator::mash(&mut self.inferences, oldargt, &realt);
            }

            self.T.insert(String::from(argn_u), realt.clone());
        }
        Ok(realt)
    }

    pub fn bind_vartype(&mut self, argn: &str, argt: &Type, line: i16
        ) -> TypeResult
    {
        vout!("bind_vartype({}, {}: {:?})\n", self.funcname, argn, argt);
        let b = self.blocks.last_mut().unwrap();
        if !b.vars.contains_key(argn) {
            let mut vdata = VarData::default();
            vdata.assignment = Some(line);
            b.vars.insert(argn.to_string(), vdata);
        } else {
            let vdata = b.vars.get_mut(argn).unwrap();
            if vdata.assignment.is_some() {
                panic!("rebinding {}. previously bound at line {}", argn, line);
            }
            vdata.assignment = Some(line);
        }

        let realt = match argt {
            &Type::Unknown => {
                let arg_typename = format!("T_local_{}", argn);
                Type::Var(Rc::new(arg_typename))
            }
            &Type::AnonVar => {
                panic!("cannot bind var to anonymous type: {}", argn);
            }
            _ => argt.clone(),
        };
        if !self.T.contains_key(argn) {
            self.T.insert(String::from(argn), realt.clone());
            return Ok(realt)
        }

        let oldargt = self.T.get(argn).unwrap();
        Inferator::mash(&mut self.inferences, oldargt, &realt)
    }

    pub fn merge_types(&mut self, a: &Type, b: &Type) -> TypeResult
    {
        Inferator::mash(&mut self.inferences, a, b)
    }

    pub fn match_pattern(&mut self, patt: &Val, valtype: &Type, lineno: i16)
    {
        match (patt, valtype) {
            (_, &Type::AnonVar) => {
                panic!("pattern value type cannot be anonymous: {:?}"
                        , patt);
            }
            (&Val::Id(ref id), _) => {
                self.bind_vartype(id, valtype, lineno);
            }
            (&Val::Nil, _) => {
                self.merge_types(
                    &Type::StrictList(Box::new(Type::Unknown)),
                    valtype,
                );
            }
            (&Val::Cons(ref head, _), &Type::StrictList(ref subt)) => {
                self.match_list_pattern(patt, subt, lineno);
            }
            (&Val::Cons(ref head, ref tail), &Type::Var(ref tvar_name)) => {
                let tvar_inner_name = format!("{}_inner", tvar_name);
                let tvar_inner = Type::Var(Rc::new(tvar_inner_name));
                self.match_list_pattern(patt, &tvar_inner, lineno);
                self.merge_types(&valtype,
                    &Type::StrictList(Box::new(tvar_inner.clone())));
            }
            (&Val::Tuple(ref flds1), &Type::Tuple(ref item_types)) => {
                for (fp, ft) in flds1.0.iter().zip(item_types.0.iter()) {
                    self.match_pattern(&fp.1, &ft.1, lineno);
                }
            }
            (&Val::Struct(ref typ1, ref flds1)
                , &Type::UserDef(ref typename2)) =>
            {
                /*
                let type_match = match typ1 {
                    &Type::Struple(ref typename1) => {
                        if typename1 != typename2 {
                            panic!("struple pattern mismatch: {} != {}"
                                , typename1, typename2);
                        }
                        let nflds1 = flds1.len();
                        let nflds2 = flds2.len();
                        if nflds1 != nflds2 {
                            panic!("struple pattern mismatch: {:?} != {:?}"
                                , patt, valtype);
                        }
                        for (fp, ft) in flds1.0.iter().zip(flds2.iter()) {
                            self.match_pattern(fp, ft, lineno);
                        }
                    }
                    _ => {
                        panic!("struple pattern type mismatch: {:?} != {:?}"
                            , patt, valtype);
                    }
                };
                */
            }
            (&Val::EnumStruct(ref typ1, ref var1, ref flds1)
                , &Type::UserDef(ref typename2)) =>
            {
            }
            (&Val::EnumToken(ref typ1, ref var1)
                , &Type::UserDef(ref typename2)) =>
            {
            }
            (&Val::Token(ref typ1)
                , &Type::UserDef(ref typename2)) =>
            {
            }
            _ => {
                let ptype = patt.get_type();
                self.merge_types(&ptype, valtype)
                    .map_err(|e| {
                        e.add_context(format!(
                            "pattern type mismatch: {:?} != {:?}"
                                , patt, valtype
                        ))
                    })
                    .unwrap();
            }
        }
    }

    pub fn match_list_pattern(&mut self, l: &Val, inner_type: &Type
        , lineno: i16)
    {
        match l {
            &Val::Cons(ref head, ref tail) => {
                self.match_pattern(head, inner_type, lineno);
                self.match_list_pattern(tail, inner_type, lineno);
            }
            &Val::Id(ref idname) => {
                let ltype = Type::StrictList(Box::new(inner_type.clone()));
                self.bind_vartype(idname, &ltype, lineno);
            }
            &Val::Nil => {}
            &Val::Wildcard => {}
            _ => {
                panic!("match_list_pattern on not a list: {:?}", l);
            }
        }
    }

    pub fn mark_usage(&mut self, name: &str, loc: &SrcLoc) -> bool
    {
        let b_opt = self.blocks.iter_mut().rev().find(|iblock| {
            iblock.vars.contains_key(name)
        });
        if b_opt.is_none() {
            panic!("cannot mark usage on undefined var: {}", name);
        }
        // safe to unwrap these 2 directly b/c we already found it above
        let b = b_opt.unwrap();
        let var_data = b.vars.get_mut(name).unwrap();
        if var_data.first_usage.is_some() {
            return false;
        }
        var_data.first_usage = Some(loc.clone());
        true
    }

    pub fn mark_failing(&mut self)
    {
        self.blocks.last_mut().unwrap().failing = true;
    }

    pub fn is_root_block(&self) -> bool
    {
        self.blocks.len() == 1
    }

    pub fn push_block(&mut self, failures: HashMap<String, Ast>)
    {
        self.blocks.push(Blockscope::new(failures));
    }

    /**
     * Pop a block off of the stack
     * Return true if the new block is the root block (only 1 left)
     */
    pub fn pop_block(&mut self) -> bool
    {
        self.blocks.pop();
        self.blocks.len() == 1
    }

    pub fn var_is_in_scope(&self, name: &str) -> bool
    {
        self.blocks.iter().any(|b| {
            b.vars.get(name)
                .map_or(false, |v| {
                    v.assignment.is_some()
                })
        })
    }

    pub fn take_current_module(&mut self) -> Option<Rc<String>>
    {
        self.module.take()
    }

    pub fn push_module(&mut self, m: Rc<String>)
    {
        if self.module.is_some() {
            panic!("cannot push {} on top of {}"
                , m, self.module.as_ref().unwrap());
        }
        self.module = Some(m);
    }

    pub fn pop_module(&mut self)
    {
        if self.module.is_none() {
        }
        self.module = None;
    }

    pub fn handles_failure(&self, name: &str) -> bool
    {
        self.blocks.iter().any(|b| {
            let optv = b.vars.get(name);
            if optv.is_none() {
                return false;
            }
            optv.unwrap().failure.is_some()
        })
    }

    pub fn get_failure(&self, name: &str) -> Option<&Ast>
    {
        for b in self.blocks.iter() {
            if b.vars.contains_key(name) {
                let v_opt: Option<&VarData> = b.vars.get(name);
                if v_opt.is_none() {
                    continue;
                }
                let v: &VarData = v_opt.as_ref().unwrap();
                return v.failure.as_ref();
            }
        }
        None
    }

    pub fn inferred_type<'a>(&'a self, typ: &'a Type) -> Type
    {
        match typ {
            &Type::Var(ref varname) => {
                match self.inferences.get(&**varname) {
                    Some(ref other_type) => {
                        self.inferred_type(other_type)
                    }
                    None => typ.clone(),
                }
            }
            &Type::StrictList(ref inner) => {
                Type::StrictList(Box::new(self.inferred_type(inner)))
            }
            &Type::Tuple(ref inners) => {
                let infers = inners.0.iter().map(|i| {
                    (i.0.clone(), self.inferred_type(&i.1))
                }).collect();
                Type::Tuple(infers)
            }
            _ => typ.clone()
        }
    }

    fn mash(inferences: &mut HashMap<Rc<String>, Type>
        , oldt: &Type, newt: &Type
        ) -> TypeResult
    {
        if oldt == newt {
            // all good
            return Ok(oldt.clone());
        }
        vout!("mash({:?}, {:?})\n", oldt, newt);
        let mtype: TypeResult = match (oldt, newt) {
            // anything is better than Unknown
            (&Type::Unknown, _) => Ok(newt.clone()),
            (_, &Type::Unknown) => Ok(oldt.clone()),
            // handle variables
            (&Type::Var(ref oldtname), &Type::Var(ref newtname)) => {
                if oldtname < newtname {
                    inferences.insert(newtname.clone(), oldt.clone());
                    Ok(oldt.clone())
                } else {
                    inferences.insert(oldtname.clone(), newt.clone());
                    Ok(newt.clone())
                }
            }
            (&Type::StrictList(ref oldit), &Type::StrictList(ref newit)) => {
                Inferator::mash(inferences, oldit, newit).and_then(|t| {
                    Ok(Type::StrictList(Box::new(t)))
                })
            }
            (&Type::Var(ref oldtname), _) => {
                inferences.insert(oldtname.clone(), newt.clone());
                Ok(newt.clone())
            }
            (_, &Type::Var(ref newtname)) => {
                inferences.insert(newtname.clone(), oldt.clone());
                Ok(oldt.clone())
            }
            (&Type::Func(ref oldargs, ref oldresult),
                    &Type::Func(ref newargs, ref newresult)
            ) => {
                let oldlen = oldargs.len();
                let newlen = newargs.len();
                if oldlen != newlen {
                    panic!("function arg count mismatch: {}!={}",
                        oldlen, newlen);
                }
                let mut masht = Vec::with_capacity(oldlen);
                for (oldit, newit) in oldargs.iter().zip(newargs.iter()) {
                    let mashit = Inferator::mash(inferences, oldit, newit)
                        .expect("function args mismatch");
                    masht.push(mashit);
                }
                let mashresult =
                    Inferator::mash(inferences, oldresult, newresult)
                        .expect("function result mismatch");
                Ok(Type::Func(masht, Box::new(mashresult)))
            }
            (_, _) => {
                Err(TypeErr::Mismatch(
                    oldt.clone(),
                    newt.clone(),
                    ))
            }
        };
        vout!("\tmashed to -> {:?}\n", mtype);
        mtype
    }


    pub fn make_call_type(&mut self, ftype: &Type, argst: &Vec<&Type>
        ) -> TypeResult
    {
        let (defargst, defresult) = Type::split_func(ftype);

        let defargslen = defargst.len();
        let argslen = argst.len();
        if argslen > defargslen {
            panic!("too many args passed to {:?}: {:?}", ftype, argst);
        }
        if argslen < defargslen {
            panic!("it's so much fun to curry, but not supported yet");
        }

        for (defargt, argt) in defargst.iter().zip(argst.iter()) {
            Inferator::mash(&mut self.inferences, defargt, argt)
                .map_err(|e| {
                    e.add_context(format!(
                        "expected function args in {}: {:?} found {:?}",
                        self.funcname, defargst, argst,
                    ))
                })
                .unwrap();
        }
        Ok(self.inferred_type(defresult))
    }
}


#[cfg(test)]
mod tests {
    use leema::infer::{Inferator};
    use leema::list;
    use leema::log;
    use leema::struple::{Struple};
    use leema::val::{Val, Type};

    use std::rc::{Rc};


#[test]
fn test_add_and_find()
{
    let mut t = Inferator::new("burritos");
    t.bind_vartype("a", &Type::Int, 18);
    assert_eq!(Type::Int, t.vartype("a").unwrap());
}

#[test]
fn test_merge_strict_list_unknown()
{
    let mut t = Inferator::new("burritos");
    let mtype = t.merge_types(
        &Type::StrictList(Box::new(Type::Unknown)),
        &Type::StrictList(Box::new(Type::Int)),
    );

    assert_eq!(Ok(Type::StrictList(Box::new(Type::Int))), mtype);
}

#[test]
fn test_merge_types_via_tvar()
{
    let mut t = Inferator::new("burritos");
    let intlist = Type::StrictList(Box::new(Type::Int));
    let unknownlist = Type::StrictList(Box::new(Type::Unknown));
    let tvar = Type::Var(Rc::new("Taco".to_string()));

    let mtype0 = t.merge_types(&unknownlist, &tvar);
    assert_eq!(Ok(unknownlist), mtype0);

    let mtype1 = t.merge_types(&intlist, &tvar);
    assert_eq!(Ok(intlist), mtype1);
}

#[test]
fn test_take_current_module()
{
    let mut t = Inferator::new("burritos");
    assert_eq!(None, t.take_current_module());
    t.push_module(Rc::new(String::from("torta")));
    assert_eq!("torta", &*t.take_current_module().unwrap());
    assert_eq!(None, t.take_current_module());
}

#[test]
fn test_match_pattern_empty_list()
{
    let mut t = Inferator::new("burritos");
    let tvar = Type::Var(Rc::new("Taco".to_string()));
    t.match_pattern(&Val::Nil, &tvar, 55);

    assert_eq!(Type::StrictList(Box::new(Type::Unknown)),
        t.inferred_type(&tvar));
}

#[test]
fn test_match_pattern_empty_and_full_lists()
{
    let mut t = Inferator::new("burritos");
    let tvar = Type::Var(Rc::new("Taco".to_string()));
    t.match_pattern(&Val::Nil, &tvar, 32);
    t.match_pattern(&list::singleton(Val::Int(5)), &tvar, 99);

    assert_eq!(Type::StrictList(Box::new(Type::Int)),
        t.inferred_type(&tvar));
}

#[test]
fn test_match_pattern_hashtag_list_inside_tuple()
{
    let mut t = Inferator::new("burritos");
    let tvar = Type::Tuple(vec![
        (None, Type::Var(Rc::new("Taco".to_string())))
    ]);
    let listpatt = Val::Struple(None, Struple::new_tuple2(
        Val::hashtag("leema".to_string()),
        Val::id("tail".to_string()),
    ));
    t.match_pattern(&listpatt, &tvar, 14);

    let exp = Type::Tuple(vec![
        (None, Type::StrictList(Box::new(Type::Hashtag))),
    ]);
    assert_eq!(exp, t.inferred_type(&tvar));
}

}
