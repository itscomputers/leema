use leema::val::{Val, Type, LibVal};
use leema::reg::{Reg};
use leema::code::{Code, CodeKey};
use leema::frame::{self, Frame};
use leema::compile::{StaticSpace};
use leema::ast;
use std::fs::File;
use std::io::{stderr, Read, Write};
use std::mem;
use std::any::{Any};
use std::fmt::{self, Display, Debug};
use std::sync::{Arc, Mutex};


pub fn int_add(fs: &mut Frame)
{
	println!("run int_add! {:?}", fs);
	let ic;
	{
		let a = fs.e.get_param(0);
		let b = fs.e.get_param(1);
		match (a,b) {
			(&Val::Int(ia), &Val::Int(ib)) => {
				ic = ia + ib;
			}
			_ => {
				panic!("wtf is all that? {:?}", (a,b));
			}
		}
	}
	fs.e.set_reg(&Reg::Result, Val::Int(ic));
}

pub fn int_sub(fs: &mut Frame)
{
	println!("run int_sub! {:?}", fs);
	let ic;
	{
		let a = fs.e.get_param(0);
		let b = fs.e.get_param(1);
		match (a,b) {
			(&Val::Int(ia), &Val::Int(ib)) => {
				ic = ia - ib;
			}
			_ => {
				panic!("wtf is all that? {:?}", (a,b));
			}
		}
	}
	fs.e.set_reg(&Reg::Result, Val::Int(ic));
}

pub fn int_mult(fs: &mut Frame)
{
	let ic;
	{
		let a = fs.e.get_param(0);
		let b = fs.e.get_param(1);
		match (a,b) {
			(&Val::Int(ia), &Val::Int(ib)) => {
				ic = ia * ib;
			}
			_ => {
				panic!("can't multiply that! {:?}", (a,b));
			}
		}
	}
	fs.e.set_reg(&Reg::Result, Val::Int(ic));
}

pub fn int_negate(fs: &mut Frame)
{
	let result;
	{
		let a = fs.e.get_param(0);
		match a {
			&Val::Int(a) => {
				result = -a;
			}
			_ => {
				panic!("can't negate a not int? {:?}", a);
			}
		}
	}
	fs.e.set_reg(&Reg::Result, Val::Int(result));
}

pub fn bool_not(fs: &mut Frame)
{
	println!("run bool_not!");
	let bnot;
	{
		let bval = fs.e.get_param(0);
		bnot = match bval {
			&Val::Bool(b) => !b,
			_ => {
				panic!("wtf is all that? {:?}", bval);
			}
		}
	}
	fs.e.set_reg(&Reg::Result, Val::Bool(bnot));
}

pub fn bool_xor(fs: &mut Frame)
{
	let result;
	{
		let va = fs.e.get_param(0);
		let vb = fs.e.get_param(1);
		match (va,vb) {
			(&Val::Bool(a), &Val::Bool(b)) => {
				result = a && !b || b && !a;
			}
			_ => {
				panic!("wtf is all that? {:?}", (va,vb));
			}
		}
	}
	fs.e.set_reg(&Reg::Result, Val::Bool(result));
}

pub fn less_than(fs: &mut Frame)
{
	let result;
	{
		let va = fs.e.get_param(0);
		let vb = fs.e.get_param(1);
		result = va < vb;
	}
	fs.e.set_reg(&Reg::Result, Val::Bool(result));
}

pub fn less_than_equal(fs: &mut Frame)
{
	let result;
	{
		let va = fs.e.get_param(0);
		let vb = fs.e.get_param(1);
		result = va <= vb;
	}
	fs.e.set_reg(&Reg::Result, Val::Bool(result));
}

pub fn equal(fs: &mut Frame)
{
	let result;
	{
		let va = fs.e.get_param(0);
		let vb = fs.e.get_param(1);
		result = va == vb;
println!("equal({}, {}) is {}", va, vb, result);
	}
	fs.e.set_reg(&Reg::Result, Val::Bool(result));
}

pub fn greater_than(fs: &mut Frame)
{
	let result;
	{
		let va = fs.e.get_param(0);
		let vb = fs.e.get_param(1);
		result = va > vb;
	}
	fs.e.set_reg(&Reg::Result, Val::Bool(result));
}

pub fn greater_than_equal(fs: &mut Frame)
{
	let result;
	{
		let va = fs.e.get_param(0);
		let vb = fs.e.get_param(1);
		result = va >= vb;
	}
	fs.e.set_reg(&Reg::Result, Val::Bool(result));
}

pub fn get_type(fs: &mut Frame)
{
	let result;
	{
		let v = fs.e.get_param(0);
		result = v.get_type();
	}
	fs.e.set_reg(&Reg::Result, Val::Type(result));
}

pub fn cout(fs: &mut Frame)
{
	{
		let va = fs.e.get_param(0);
		print!("{}", va);
	}
	fs.e.set_reg(&Reg::Result, Val::Void);
}

pub fn cerr(fs: &mut Frame)
{
	{
		let va = fs.e.get_param(0);
		write!(stderr(), "{}", va);
	}
	fs.e.set_reg(&Reg::Result, Val::Void);
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
		let fnval = fs.e.get_param(0);
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
		Err(_) => Val::Failure,
	};
	fs.e.set_reg(&Reg::Result, openf);
}

pub fn stream_read_file(fs: &mut Frame)
{
	let mut input = "".to_string();
	{
		let mut streamval = fs.e.get_mut_param(0);
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
	fs.e.set_reg(&Reg::Result, Val::Str(Arc::new(input)));
}

pub fn define_prefab(ss: &mut StaticSpace)
{
	// set up static space
	ss.define_func(Arc::new("int_add".to_string()), Type::Int,
		vec![Type::Int, Type::Int], Code::Rust(int_add),
		);
	ss.define_func(Arc::new("int_sub".to_string()), Type::Int,
		vec![Type::Int, Type::Int], Code::Rust(int_sub),
		);
	ss.define_func(Arc::new("int_mult".to_string()), Type::Int,
		vec![Type::Int, Type::Int], Code::Rust(int_mult),
		);
	ss.define_func(Arc::new("negate".to_string()), Type::Int,
		vec![Type::Int], Code::Rust(int_negate),
		);
	ss.define_func(Arc::new("bool_not".to_string()), Type::Bool,
		vec![Type::Bool], Code::Rust(bool_not),
		);
	ss.define_func(Arc::new("xor".to_string()), Type::Bool,
		vec![Type::Bool, Type::Bool], Code::Rust(bool_xor),
		);
	ss.define_func(Arc::new("less_than".to_string()), Type::Bool,
		vec![Type::Int, Type::Int], Code::Rust(less_than),
		);
	ss.define_func(Arc::new("less_than_equal".to_string()), Type::Bool,
		vec![Type::Int, Type::Int], Code::Rust(less_than_equal),
		);
	ss.define_func(Arc::new("equal".to_string()), Type::Bool,
		vec![Type::Int, Type::Int], Code::Rust(equal),
		);
	ss.define_func(Arc::new("greater_than".to_string()), Type::Bool,
		vec![Type::Int, Type::Int], Code::Rust(greater_than),
		);
	ss.define_func(Arc::new("greater_than_equal".to_string()), Type::Bool,
		vec![Type::Int, Type::Int], Code::Rust(greater_than_equal),
		);
	ss.define_func(Arc::new("type".to_string()), Type::Kind,
		vec![Type::Any], Code::Rust(get_type),
		);
	ss.define_func(Arc::new("cout".to_string()), Type::Void,
		vec![Type::Str], Code::Rust(cout));
	ss.define_func(Arc::new("file_read".to_string()),
		Type::Lib("File".to_string()),
		vec![Type::Str], Code::Rust(file_read)
		);
	ss.define_func(Arc::new("stream_read".to_string()),
		Type::Void,
		vec![Type::Lib("File".to_string())],
		Code::Rust(stream_read_file),
		);
}

pub fn new_staticspace() -> StaticSpace
{
	let mut ss = StaticSpace::new();
	define_prefab(&mut ss);
	ss
}
