use leema::code::Code;
use leema::log;
use leema::lstr::Lstr;
use leema::rsrc;
use leema::val::Val;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;


pub fn file_open(_ctx: rsrc::IopCtx) -> rsrc::Event
{
    vout!("file_open()\n");
    rsrc::Event::Result(Val::Void, None)
}

pub fn file_read_file(mut ctx: rsrc::IopCtx) -> rsrc::Event
{
    vout!("file_read_file()\n");
    let pathval = ctx.take_param(0).unwrap();
    let path = Path::new(pathval.str());
    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).expect("read_to_string failure");
    rsrc::Event::Result(Val::Str(Lstr::from(s)), None)
}

pub fn file_write(_ctx: rsrc::IopCtx) -> rsrc::Event
{
    vout!("file_write()\n");
    rsrc::Event::Result(Val::Void, None)
}

pub fn file_write_file(mut ctx: rsrc::IopCtx) -> rsrc::Event
{
    vout!("file_write_file()\n");
    let pathval = ctx.take_param(0).unwrap();
    let output = ctx.take_param(1).unwrap();
    let path = Path::new(pathval.str());
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();
    f.write_all(output.str().as_bytes())
        .expect("write_all failure");
    rsrc::Event::Result(Val::Void, None)
}

pub fn load_rust_func(func_name: &str) -> Option<Code>
{
    match func_name {
        "read_file" => Some(Code::Iop(file_read_file, None)),
        "write_file" => Some(Code::Iop(file_write_file, None)),
        _ => None,
    }
}
