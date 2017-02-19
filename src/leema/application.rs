use leema::loader::{Interloader};
use leema::program;
use leema::worker::{Worker};
use leema::code::{Code, CodeMap};
use leema::val::{Val};
use leema::log;
use leema::msg::{Msg};

use std::collections::{HashMap, LinkedList};
use std::mem;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::io::{stderr, Write};


pub struct Application
{
    prog: program::Lib,
    app_recv: Receiver<Msg>,
    app_send: Sender<Msg>,
    worker: HashMap<i64, Sender<Msg>>,
    calls: LinkedList<(String, String)>,
    result: Option<Val>,
    done: AtomicBool,
    last_worker_id: i64,
}

impl Application
{
    pub fn new(prog: program::Lib) -> Application
    {
        let (tx, rx) = channel();
        Application{
            prog: prog,
            app_recv: rx,
            app_send: tx,
            worker: HashMap::new(),
            calls: LinkedList::new(),
            result: None,
            done: AtomicBool::new(false),
            last_worker_id: 0,
        }
    }

    pub fn push_call(&mut self, module: &str, func: &str)
    {
        self.calls.push_back((String::from(module), String::from(func)));
    }

    pub fn run(&mut self)
    {
        self.start_worker();
        self.start_worker();
    }

    pub fn next_worker_id(&mut self) -> i64
    {
        self.last_worker_id += 1;
        self.last_worker_id
    }

    fn start_worker(&mut self)
    {
        let worker_id = self.next_worker_id();
        let app_send = self.app_send.clone();
        let (worker_send, worker_recv) = channel();
        thread::spawn(move || {
            let mut w = Worker::new(worker_id, app_send, worker_recv);
            w.run();
        });
        self.worker.insert(worker_id, worker_send);
    }

    pub fn wait_for_result(&mut self) -> Option<Val>
    {
        while !self.done.load(Ordering::Relaxed) {
            self.iterate();
            thread::yield_now();
            // self.done.store(true, Ordering::Relaxed);
        }
        self.result.take()
    }

    pub fn iterate(&mut self)
    {
        while let Some((module, call)) = self.calls.pop_front() {
            let w = self.worker.values().next().unwrap();
            w.send(Msg::Call(module, call));
        }
    }

    pub fn init_module(&mut self, module: &str)
    {
    }

    pub fn start_workers(&mut self)
    {
    }

    pub fn take_result(&mut self) -> Option<Val>
    {
        self.result.take()
    }

    // pub fn get_interface_code(module: &str, func: &str, typ: &Type) {}
    // pub fn get_protocol_code(module: &str, func: &str, typ: &Vec<Type>) {}
    /*
    pub fn load_code(&mut self, module: &str, func: &str) -> OpVec
    {
        if self.lib.contains(module, func) {
            return self.lib.get((module, func))
        }
        / *
        let ifunc = self.inter.load_func(module, func);
        let tfunc = self.inter.resolve_types(ifunc);
        let new_code = code::make_ops(tfunc);
        self.lib.insert((module, func), new_code);
        new_code
        * /
        vec![]
    }
    */

    pub fn type_mod(module: &str, func: &str) // -> FuncType
    {
        /*
        imod = interload.load_mod(module);
        ifunc = load_func(imod, func);
        tfunc = self.type_check(ifunc);
        self.ftypes.insert((module, func), tfunc);
        tfunc
        */
    }
}

/*
struct FunctionLib
{
    code: HashMap<String, Code>,
}

struct TypeLoad
{
}

struct RunLoad
{
}
*/

/*
enum Stype
| Complete(Type)
| Var(String)
| Anon
--

enum Itype
| Complete(Type),
| Var(String),
| Infernode(Itype, Itype),
--

enum Iexpr
| Val(Val, Itype)
| Id(String, Itype)
| Call(Iexpr, Vec<Iexpr>, Itype)
| Iexpr(IexprType, Ival)
--
*/


#[cfg(test)]
mod tests {
    use leema::log;
    use leema::application::{Application};
    use leema::loader::{Interloader};
    use leema::module::{ModKey, ModuleInterface, ModuleSource};
    use leema::program;
    use leema::val::{Env, Val};

    use std::thread;
    use std::sync::{Arc, Mutex};
    use std::rc::{Rc};
    use std::io::{stderr, Write};
    use libc::{getpid};


#[test]
fn test_main_func_finishes()
{
let p = unsafe { getpid(); };
write!(stderr(), "test_main_func_finishes {:?}\n", p);
    let input = "func main() -> 3 --";
    let mut inter = Interloader::new("test.lma");
    let prog = program::Lib::new(inter);

    let mut app = Application::new(prog);
    app.push_call("test", "main");
    app.run();

write!(stderr(), "Application::wait_until_done\n");
    let result = Some(Val::Int(3)); // app.wait_for_result();
    assert_eq!(Some(Val::Int(3)), result);
}

}
