use leema::val::{Val};
use std::fmt;
use std::io::{Write};

#[derive(PartialEq)]
#[derive(Eq)]
#[derive(PartialOrd)]
#[derive(Ord)]
#[derive(Clone)]
pub enum Ireg {
    Reg(i8),
    Sub(i8, Box<Ireg>),
}

impl Ireg
{
    pub fn sub(&self, newsub: i8) -> Ireg
    {
        match self {
            &Ireg::Reg(r) => Ireg::Sub(r, Box::new(Ireg::Reg(newsub))),
            &Ireg::Sub(r, ref s) => Ireg::Sub(r, Box::new(s.sub(newsub))),
        }
    }

    pub fn is_primary(&self) -> bool
    {
        match self {
            &Ireg::Reg(_) => true,
            &Ireg::Sub(_, _) => false,
        }
    }

    pub fn get_primary(&self) -> i8
    {
        match self {
            &Ireg::Reg(r) => r,
            &Ireg::Sub(r, _) => r,
        }
    }

    pub fn get_sub(&self) -> &Ireg
    {
        match self {
            &Ireg::Sub(_, ref s) => &*s,
            &Ireg::Reg(_) => panic!("Cannot get sub from Ireg::Reg"),
        }
    }
}

impl fmt::Display for Ireg
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self {
            &Ireg::Reg(r) => write!(f, ".{}", r),
            &Ireg::Sub(r, ref s) => write!(f, ".{}{}", r, s),
        }
    }
}

impl fmt::Debug for Ireg
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{}", self)
    }
}

pub trait Iregistry
{
    fn ireg_get(&self, r: &Ireg) -> &Val;
    fn ireg_get_mut(&mut self, r: &Ireg) -> &mut Val;
    fn ireg_set(&mut self, r: &Ireg, v: Val);
}

#[derive(PartialEq)]
#[derive(Eq)]
#[derive(PartialOrd)]
#[derive(Ord)]
#[derive(Clone)]
pub enum Reg {
    Params,
    Param(Ireg),
    Reg(Ireg),
    Lib,
    Void,
    Undecided,
}

impl Reg
{
    pub fn sub(&self, sub: i8) -> Reg
    {
        match self {
            &Reg::Params => {
                Reg::Param(Ireg::Reg(sub))
            }
            &Reg::Param(ref ir) => {
                Reg::Param(ir.sub(sub))
            }
            &Reg::Reg(ref r) => {
                Reg::Reg(r.sub(sub))
            }
            &Reg::Void => Reg::Void, 
            _ => {
                panic!("Can't make a subreg for {:?}", self);
            }
        }
    }

    pub fn new_param(p: i8) -> Reg
    {
        Reg::Param(Ireg::Reg(p))
    }

    pub fn new_reg(p: i8) -> Reg
    {
        Reg::Reg(Ireg::Reg(p))
    }

    pub fn is_primary(&self) -> bool
    {
        match self {
            &Reg::Params => true,
            &Reg::Reg(Ireg::Reg(_)) => true,
            _ => false,
        }
    }

    pub fn is_sub(&self) -> bool
    {
        match self {
            &Reg::Param(_) => true,
            &Reg::Reg(Ireg::Sub(_, _)) => true,
            _ => false,
        }
    }

    pub fn get_sub(&self) -> &Ireg
    {
        match self {
            &Reg::Param(ref s) => s,
            &Reg::Reg(Ireg::Sub(_, ref s)) => s,
            _ => panic!("cannot get sub from other register: {:?}", self),
        }
    }
}

impl fmt::Display for Reg
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self {
            &Reg::Param(ref r) => write!(f, "Param{}", r),
            &Reg::Reg(ref r) => write!(f, "Reg{}", r),
            &Reg::Params => write!(f, "Reg::Params"),
            &Reg::Lib => write!(f, "Reg::Lib"),
            &Reg::Void => write!(f, "Reg::Void"),
            &Reg::Undecided => write!(f, "Reg::Undecided"),
        }
    }
}

impl fmt::Debug for Reg
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{}", self)
    }
}
