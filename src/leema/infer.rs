
use leema::log;
use leema::val::{Val, Type, SrcLoc};

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Keys;
use std::io::{stderr, Write};
use std::rc::{Rc};


#[derive(Debug)]
pub struct Blockscope
{
    vars: HashSet<String>,
    first_usage: HashMap<String, SrcLoc>,
    failing: bool,
}

impl Blockscope
{
    pub fn new() -> Blockscope
    {
        Blockscope{
            vars: HashSet::new(),
            first_usage: HashMap::new(),
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
}

impl<'b> Inferator<'b>
{
    pub fn new(funcname: &'b str) -> Inferator
    {
        Inferator{
            funcname: funcname,
            T: HashMap::new(),
            blocks: vec![Blockscope::new()],
            inferences: HashMap::new(),
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

    pub fn bind_vartype(&mut self, argn: &str, argt: &Type) -> Option<Type>
    {
        let b = self.blocks.last_mut().unwrap();
        if !b.vars.contains(argn) {
            b.vars.insert(argn.to_string());
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
            return Some(realt)
        }

        let oldargt = self.T.get(argn).unwrap();
        Inferator::mash(&mut self.inferences, oldargt, &realt)
    }

    pub fn merge_types(&mut self, a: &Type, b: &Type) -> Option<Type>
    {
        Inferator::mash(&mut self.inferences, a, b)
    }

    pub fn match_pattern(&mut self, patt: &Val, valtype: &Type)
    {
        match (patt, valtype) {
            (_, &Type::AnonVar) => {
                panic!("pattern value type cannot be anonymous: {:?}"
                        , patt);
            }
            (&Val::Id(ref id), _) => {
                self.bind_vartype(id, valtype);
            }
            (&Val::Tuple(ref p_items), &Type::Tuple(ref t_items)) => {
                if p_items.len() != t_items.len() {
                    panic!("tuple pattern size mismatch: {:?} != {:?}"
                        , p_items, t_items);
                }
                for (pi, ti) in p_items.iter().zip(t_items.iter()) {
                    self.match_pattern(pi, ti);
                }
            }
            (&Val::Nil, _) => {
                self.merge_types(
                    &Type::StrictList(Box::new(Type::Unknown)),
                    valtype,
                );
            }
            (&Val::Cons(ref head, _), &Type::StrictList(ref subt)) => {
                self.match_list_pattern(patt, subt);
            }
            (&Val::Cons(ref head, ref tail), &Type::Var(ref tvar_name)) => {
                let tvar_inner_name = format!("{}_inner", tvar_name);
                let tvar_inner = Type::Var(Rc::new(tvar_inner_name));
                self.match_list_pattern(patt, &tvar_inner);
                self.merge_types(&valtype,
                    &Type::StrictList(Box::new(tvar_inner.clone())));
            }
            (&Val::Struct(ref typ1, ref flds1),
                &Type::Struct(ref typename2)
            ) => {
                let type_match = match typ1 {
                    &Type::Struct(ref typename1) => {
                        typename1 == typename2 // && nflds1 == nflds2
                    }
                    _ => {
                        panic!("struct pattern type mismatch: {:?} != {:?}"
                            , patt, valtype);
                    }
                };
                for fld in flds1 {
                    // do something w/ flds1
                }
            }
            _ => {
                let ptype = patt.get_type();
                let mtype = self.merge_types(&ptype, valtype);
                if mtype.is_none() {
                    panic!("pattern type mismatch: {:?} != {:?}"
                        , patt, valtype);
                }
            }
        }
    }

    pub fn match_list_pattern(&mut self, l: &Val, inner_type: &Type)
    {
        match l {
            &Val::Cons(ref head, ref tail) => {
                self.match_pattern(head, inner_type);
                self.match_list_pattern(tail, inner_type);
            }
            &Val::Id(ref idname) => {
                let ltype = Type::StrictList(Box::new(inner_type.clone()));
                self.bind_vartype(idname, &ltype);
            }
            &Val::Nil => {}
            &Val::Wildcard => {}
            &Val::Loc(ref lv, _) => {
                self.match_list_pattern(lv, inner_type);
            }
            _ => {
                panic!("match_list_pattern on not a list: {:?}", l);
            }
        }
    }

    pub fn mark_usage(&mut self, name: &str, loc: &SrcLoc)
    {
        let b = self.blocks.last_mut().unwrap();
        if !b.first_usage.contains_key(name) {
            b.first_usage.insert(name.to_string(), loc.clone());
        }
    }

    pub fn mark_failing(&mut self)
    {
        self.blocks.last_mut().unwrap().failing = true;
    }

    pub fn push_block(&mut self)
    {
        self.blocks.push(Blockscope::new());
    }

    pub fn pop_block(&mut self)
    {
        self.blocks.pop();
    }

    pub fn var_is_in_scope(&self, name: &str) -> bool
    {
        self.blocks.iter().any(|b| {
            b.vars.contains(name)
        })
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
                let infers = inners.iter().map(|i| {
                    self.inferred_type(i)
                }).collect();
                Type::Tuple(infers)
            }
            _ => typ.clone()
        }
    }

    fn mash(inferences: &mut HashMap<Rc<String>, Type>
            , oldt: &Type, newt: &Type) -> Option<Type>
    {
        if oldt == newt {
            // all good
            return Some(oldt.clone());
        }
        vout!("mash({:?}, {:?})\n", oldt, newt);
        let mtype = match (oldt, newt) {
            // anything is better than Unknown
            (&Type::Unknown, _) => Some(newt.clone()),
            (_, &Type::Unknown) => Some(oldt.clone()),
            // handle variables
            (&Type::Var(ref oldtname), &Type::Var(ref newtname)) => {
                if oldtname < newtname {
                    inferences.insert(newtname.clone(), oldt.clone());
                    Some(oldt.clone())
                } else {
                    inferences.insert(oldtname.clone(), newt.clone());
                    Some(newt.clone())
                }
            }
            (&Type::StrictList(ref oldit), &Type::StrictList(ref newit)) => {
                Inferator::mash(inferences, oldit, newit).and_then(|t| {
                    Some(Type::StrictList(Box::new(t)))
                })
            }
            (&Type::Var(ref oldtname), _) => {
                inferences.insert(oldtname.clone(), newt.clone());
                Some(newt.clone())
            }
            (_, &Type::Var(ref newtname)) => {
                inferences.insert(newtname.clone(), oldt.clone());
                Some(oldt.clone())
            }
            (&Type::Tuple(ref oldi), &Type::Tuple(ref newi)) => {
                let oldlen = oldi.len();
                if oldlen != newi.len() {
                    panic!("tuple size mismatch: {:?}!={:?}", oldt, newt);
                }
                let mut masht = Vec::with_capacity(oldlen);
                for (oldit, newit) in oldi.iter().zip(newi.iter()) {
                    match Inferator::mash(inferences, oldit, newit) {
                        Some(mashit) => {
                            masht.push(mashit);
                        }
                        None => {
                            panic!("tuple type mismatch: {:?} != {:?}",
                                oldt, newt);
                        }
                    }
                }
                Some(Type::Tuple(masht))
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
                    match Inferator::mash(inferences, oldit, newit) {
                        Some(mashit) => {
                            masht.push(mashit);
                        }
                        None => {
                            panic!("function args mismatch: {:?} != {:?}"
                                , oldargs, newargs);
                        }
                    }
                }
                let mashresult =
                    match Inferator::mash(inferences, oldresult, newresult) {
                        Some(mr) => mr,
                        None => {
                            panic!("function result mismatch: {:?} != {:?}"
                                , oldresult, newresult);
                        }
                    };
                Some(Type::Func(masht, Box::new(mashresult)))
            }
            (&Type::Struct(ref sname), _) if **sname == *newt => {
                Some(oldt.clone())
            }
            (_, &Type::Struct(ref sname)) if *oldt == **sname => {
                Some(newt.clone())
            }
            (_, _) => {
                println!("type mismatch: {:?} != {:?}", oldt, newt);
                None
            }
        };
        vout!("mashed to -> {:?}\n", mtype);
        mtype
    }



    pub fn make_call_type(&mut self, ftype: &Type, argst: &Vec<&Type>) -> Type
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
            if Inferator::mash(&mut self.inferences, defargt, argt).is_none() {
                return Type::Error(Rc::new(
                    format!("expected function args in {}: {:?} found {:?}",
                    self.funcname, defargst, argst)
                ));
            }
        }
        self.inferred_type(defresult)
    }
}


#[cfg(test)]
mod tests {
    use leema::infer::{Inferator};
    use leema::list;
    use leema::log;
    use leema::module::{ModKey};
    use leema::val::{Val, Type};

    use std::rc::{Rc};
    use std::io::{stderr, Write};
    use std::collections::{HashMap};


#[test]
fn test_add_and_find()
{
    let mut t = Inferator::new("burritos");
    t.bind_vartype("a", &Type::Int);
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

    assert_eq!(Some(Type::StrictList(Box::new(Type::Int))), mtype);
}

#[test]
fn test_merge_types_via_tvar()
{
    let mut t = Inferator::new("burritos");
    let intlist = Type::StrictList(Box::new(Type::Int));
    let unknownlist = Type::StrictList(Box::new(Type::Unknown));
    let tvar = Type::Var(Rc::new("Taco".to_string()));

    let mtype0 = t.merge_types(&unknownlist, &tvar);
    assert_eq!(Some(unknownlist), mtype0);

    let mtype1 = t.merge_types(&intlist, &tvar);
    assert_eq!(Some(intlist), mtype1);
}

#[test]
fn test_match_pattern_empty_list()
{
    let mut t = Inferator::new("burritos");
    let tvar = Type::Var(Rc::new("Taco".to_string()));
    t.match_pattern(&Val::Nil, &tvar);

    assert_eq!(Type::StrictList(Box::new(Type::Unknown)),
        t.inferred_type(&tvar));
}

#[test]
fn test_match_pattern_empty_and_full_lists()
{
    let mut t = Inferator::new("burritos");
    let tvar = Type::Var(Rc::new("Taco".to_string()));
    t.match_pattern(&Val::Nil, &tvar);
    t.match_pattern(&list::singleton(Val::Int(5)), &tvar);

    assert_eq!(Type::StrictList(Box::new(Type::Int)),
        t.inferred_type(&tvar));
}

#[test]
fn test_match_pattern_hashtag_list_inside_tuple()
{
    let mut t = Inferator::new("burritos");
    let tvar = Type::Tuple(vec![
        Type::Var(Rc::new("Taco".to_string()))
    ]);
    let listpatt = Val::Tuple(vec![list::cons(
        Val::hashtag("leema".to_string()),
        Val::id("tail".to_string())
    )]);
    t.match_pattern(&listpatt, &tvar);

    let exp = Type::Tuple(vec![
        Type::StrictList(Box::new(Type::Hashtag)),
    ]);
    assert_eq!(exp, t.inferred_type(&tvar));
}

}
