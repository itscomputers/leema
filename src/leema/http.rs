use leema::application::{AppCaller};
use leema::lstr::Lstr;

use std::thread;

use futures::Future;
use futures::future;
use futures::sync::oneshot::Canceled;
use hyper::{Body, Request, Response, Server};
use hyper::service::service_fn;


type HttpFut = Future<Item = Response<Body>, Error = Canceled>;
type BoxFut = Box<Future<Item = Response<Body>, Error = Canceled> + Send>;

pub fn spawn_thread(app: AppCaller) -> thread::JoinHandle<()>
{
    thread::spawn(move || {
        run(app);
    })
}

pub fn run(app: AppCaller)
{
    let addr = ([0, 0, 0, 0], 3000).into();

    let new_svc = move || {
        let iapp = app.clone();
        service_fn(move |req| {
            handle_request(iapp.clone(), req)
        })
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    let http_result = ::hyper::rt::run(server);
    println!("hyper finished with: {:?}", http_result);
}

pub fn handle_request(caller: AppCaller, req: Request<Body>) -> BoxFut
{
    println!("handle_request({:?})", req);
    let response_future: BoxFut = Box::new(
        caller.push_call(
            &Lstr::Sref("http"), &Lstr::Sref("test_handle")
        )
        .and_then(|v| {
            let msg = format!("{}", v);
            println!("response msg: {}", msg);
            // future::ok(Response::new(Body::from(msg)))
            future::ok(Response::new(Body::from(msg)))
        })
        .map_err(|e| {
            println!("request error: {:?}", e);
            e
        })
    );
    response_future
}

struct LeemaService
{
    app: AppCaller,
}
