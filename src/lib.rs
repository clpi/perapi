mod utils;

use console_error_panic_hook::{
    hook, set_once as set_hook
};
use serde_json::{json, from_str, Value, Result, to_string, to_value};
use worker::{
    Request,

    console_log, Date, Storage, Result as WorkerResult,
    Response, ResponseBody, Env, Context, Router, FormEntry,

};

fn log_req(req: &Request) {
    console_log!(
        "{} - [{}], @ {:?}, within {}",
        Date::now().to_string(),
        req.path(),
        req.cf()
            .coordinates(),
        req.cf()
            .region()
            .unwrap_or("unknown region".into())
    );
}

impl From<String> for WorkerResult<Response> {
    fn from(s: String) -> Self {
        let res: Vec<u8> = s.as_bytes().to_vec();
        let resp: ResponseBody = ResponseBody::Body(res);
        Response::from_body(resp)
    }
}

#[worker::event(fetch)]
pub async fn mainl(
    r: Request, 
    env: Env, 
    c: Context
) -> WorkerResult<Response> {
    log_req(&r);
    set_hook();
    let routes = Router::new();
    routes
        .get_async("/", |mut _req, _ctx| async move { Response::Ok("Hello from workers!" )})
        .post_async("/tag/:tagname", |mut req, ctx| async move {
            if let Some(name) = ctx.param("field") {
                let form = req.form_data().await?;
                match form.get(name) {
                    Some(FormEntry::Field(val)) => {
                        return Response::from_json(&json!({ name: val }));
                    },
                    Some(FormEntry::File(_f)) => {
                        return Response::error("Field param must be raw text", 422);
                    },
                    None => return Response::error("Bad request", 400)
                }
            }
            Response::error("Bad request", 400)
        })
        .get_async("/worker-version", |mut _req, ctx| async move {
            let v = ctx.var("WORKERS_RS_VERSION")?.to_string();
            return Response::ok(v);
        })
            ;
    routes.run(req, env);

}
