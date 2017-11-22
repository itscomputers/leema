
use leema::code::{Code};
use leema::log;
use leema::rsrc;
use leema::val::{Val};

use std::fs::{File, OpenOptions};
use std::io::{self, stderr, Read, Write};
use std::path::{Path};


pub fn file_open(ctx: rsrc::IopCtx) -> rsrc::Event
{
    vout!("file_open()\n");
    rsrc::Event::Success(Val::Void, None)
}

pub fn file_read_file(mut ctx: rsrc::IopCtx) -> rsrc::Event
{
    vout!("file_read_file()\n");
    let pathval = ctx.take_param(0).unwrap();
    let path = Path::new(pathval.str());
    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s);
    rsrc::Event::Success(Val::new_str(s), None)
}

pub fn file_write(ctx: rsrc::IopCtx) -> rsrc::Event
{
    vout!("file_write()\n");
    rsrc::Event::Success(Val::Void, None)
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
        .open(path).unwrap();
    f.write_all(output.str().as_bytes());
    rsrc::Event::Success(Val::Void, None)
}

pub fn load_rust_func(func_name: &str) -> Option<Code>
{
    match func_name {
        "read_file" => Some(Code::Iop(file_read_file, None)),
        "write_file" => Some(Code::Iop(file_write_file, None)),
        _ => None,
    }
}
