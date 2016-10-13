

struct Worker
{
    code: HashMap<(String, String), Code>,
    app_channel: Channel<CodeRequest>,
    code_request_idx: u64,
}

impl Worker
{
    pub fn new(app_ch: Channel<AppRequest>) -> Worker
    {
        Worker{
            code: HashMap::new(),
            app_channel: app_ch,
            code_request_idx: 0,
        }
    }

    pub fn call_func(&mut self, module: &str, func: &str)
    {
        push_frame(get_code(module, func))
    }

    pub fn get_code(&mut self, module: &str, func: &str)
    {
        c = code.find(module, func);
        if c.is_none() {
            i = self.new_code_request();
            c = self.app_channel.push(CodeRequest(i, module, func));
            frame.wait_on_code(i)
        }
        c
    }

    pub fn new_code_request(&mut self) -> u64
    {
        let idx = self.code_request_idx;
        self.code_request_idx += 1;
        idx
    }
}

pub struct Application
{
    inter: Interloader,
    lib: CodeMap,
    result: Val,
}

impl Application
{
    pub fn new(i: Interload) -> Application
    {
        Application{
            inter: i,
            lib: HashMap::new(),
            result: Val::Void,
        }
    }

    pub fn push_call(&mut self, module: &str, func: &str)
    {
    }

    pub fn run(&mut self) -> Val
    {
    }

    pub fn init_module(&mut self, module: &str)
    {
    }

    pub fn start_workers(&mut self)
    {
    }

    pub fn take_result(&mut self) -> Val
    {
    }

    pub fn get_interface_code(module: &str, func: &str, typ: &Type) {}
    pub fn get_protocol_code(module: &str, func: &str, typ: &Vec<Type>) {}
    pub fn load_code(&mut self, module: &str, func: &str)
    {
        if self.lib.contains(module, func) {
            return self.lib.get((module, func))
        }
        let ifunc = self.inter.load_func(module, func);
        let tfunc = self.inter.resolve_types(ifunc);
        let new_code = code::make_ops(tfunc);
        self.lib.insert((module, func), new_code);
        new_code
    }

    pub fn make_code(func: &Iexpr) -> Code
    {
        imod = load_imod(module);
        ifunc = get_ifunc(imod, func);
        store_resolved_interface(module, func, tfunc);
        cfunc = tfunc_to_code(tfunc);
        cfunc
    }

    pub fn type_mod(module: &str, func: &str) -> FuncType
    {
        imod = interload.load_mod(module);
        ifunc = load_func(imod, func);
        tfunc = self.type_check(ifunc);
        self.ftypes.insert((module, func), tfunc);
        tfunc
    }
}

struct ModuleLib
{
    imports: HashSet<String>,
    lib: HashMap<String, Module>,
}

struct FunctionLib
{
    code: HashMap<String, Code>,
}

struct Interloader
{
}

struct TypeLoad
{
}

struct RunLoad
{
}

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
