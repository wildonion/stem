



use salvo::prelude::*;
use salvo::{FlowCtrl, Request, Response, Depot};
use crate::*;
use serde::{Serialize, Deserialize};
use interfaces::ShaHasher;


#[derive(Clone)]
pub struct Context{
    pub apiCount: Arc<tokio::sync::Mutex<ApiCount>>
}

#[derive(Serialize, Deserialize)]
pub struct ApiCount{
    pub map: HashMap<String, (String, AtomicU64)>,
}


#[handler]
pub async fn greeting(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
    ctrl: &mut FlowCtrl
){

    res.render("hello from greeting");
}


#[handler]
pub async fn countCall(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
    ctrl: &mut FlowCtrl
){
    let ctx = depot.get::<Context>("ctx").unwrap();
    let mut apiCount = ctx.apiCount.lock().await;
    (*apiCount).map.entry(
        req.uri().to_string()
    ).and_modify(|(reqId, count)| {
        let current = count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    })
    .or_insert(
        (
            {
                let now = chrono::Local::now().timestamp();
                let mut nowString = format!("{}", now);
                nowString.hashMe();
                nowString
            },
            AtomicU64::new(1)
        )
    );

}

#[handler]
pub async fn report(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
    ctrl: &mut FlowCtrl
){
    let ctx = depot.get::<Context>("ctx").unwrap();
    let reportCall = ctx.apiCount.lock().await;
    let reportMap = &reportCall.map; // can't move out of Mutex so we're borrowing the map in here
    res.render(Json(reportMap));

}


#[handler]
pub async fn getAllEntitiesHandler(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
    ctrl: &mut FlowCtrl
){}



pub async fn buildRouters() -> Router{
    
    let routers = Router::with_path("/report/")
        .hoop(countCall)
        .push(
            Router::with_path("greeting")
            .get(greeting)
        )
        .push(
            Router::with_path("report")
            .get(report)
        );

    routers

}