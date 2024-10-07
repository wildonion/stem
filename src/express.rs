

use crate::*;
use cors::Cors;
use handler::HoopedHandler;
use prelude::{max_concurrency, Compression, CompressionLevel};
use salvo::*;
use salvo::http::Method;
use test::ResponseExt;



type Middleware<R> = fn(&mut Request, &mut Response, &mut FlowCtrl, &mut Depot) -> R;

#[derive(Clone)]
struct Express{
    pub router: std::sync::Arc<Router>, // the building blocks of all apis in salvo
}

#[derive(Clone)]
struct AppCtx{

}

impl Express{
    
    pub fn new(app_ctx: AppCtx) -> Self{
        let cors = Cors::new()
            .allow_origin("*")
            .allow_methods(vec![Method::GET, Method::POST, Method::DELETE, Method::PATCH, Method::PUT])
            .into_handler();

        let router = std::sync::Arc::new(
            Router::with_path("")
                    .hoop(Compression::new().enable_brotli(CompressionLevel::Fastest))
                    .hoop(affix_state::inject(app_ctx))
                    .hoop(cors)
                    .hoop(max_concurrency(10))
        );
        Self { router }
    }

    // use this function to take the ownership of the inner value inside the Arc
    pub fn take_inner_router(self) -> Option<Router>{
        match std::sync::Arc::try_unwrap(self.router){
            Ok(router) => {
                Some(router)
            },
            Err(e) => { // can't take it just return the Arc<Router>
                None
            }
        }
    }

    // with dynamic dispatch we can pass a method which impls the Handler trait
    // to one of the router method.
    // every api in salvo must be bounded to salvo Handler trait
    // every middleware in salvo is an api handler which calles before all apis
    pub fn post<T, F, R>(&mut self, path: &str, middlewares: Vec<Middleware<R>>, callback: T) where 
        T: Fn(&mut Request, &mut Response, &mut FlowCtrl, &mut Depot) -> R + Clone + Handler + Send + Sync + 'static,
        F: Fn(&mut Request, &mut Response, &mut FlowCtrl, &mut Depot) -> R + Handler + Send + Sync + 'static,
        R: std::future::Future<Output = ()>{

            let mut hoopRouters = middlewares
                .into_iter()
                .map(|m| Router::new().hoop(m))
                .collect::<Vec<Router>>();
            
            let mut newRouter = Router::with_path(path).post(callback);
            if let Some(inner_router) = self.clone().take_inner_router(){
                newRouter
                    .push(inner_router) // push the current router to the new one
                    .append(&mut hoopRouters); // also add all middlewares to it
            }
            
    }

    pub fn get<F, R>(&mut self, path: &str, middlewares: Vec<F>, callback: F) where 
        F: Fn(&mut Request, &mut Response, &mut FlowCtrl, &mut Depot) -> R + Clone + Handler + Send + Sync + 'static,
        R: std::future::Future<Output = ()>{

            let mut hoopRouters = middlewares
                .into_iter()
                .map(|m| Router::new().hoop(m))
                .collect::<Vec<Router>>();
            
            let mut newRouter = Router::with_path(path).get(callback);
            if let Some(inner_router) = self.clone().take_inner_router(){
                newRouter
                    .push(inner_router) // push the current router to the new one
                    .append(&mut hoopRouters); // also add all middlewares to it
            }
            
    }

    pub fn put<F, R>(&mut self, path: &str, middlewares: Vec<F>, callback: F) where 
        F: Fn(&mut Request, &mut Response, &mut FlowCtrl, &mut Depot) -> R + Clone + Handler + Send + Sync + 'static,
        R: std::future::Future<Output = ()>{

            let mut hoopRouters = middlewares
                .into_iter()
                .map(|m| Router::new().hoop(m))
                .collect::<Vec<Router>>();
            
            let mut newRouter = Router::with_path(path).put(callback);
            if let Some(inner_router) = self.clone().take_inner_router(){
                newRouter
                    .push(inner_router) // push the current router to the new one
                    .append(&mut hoopRouters); // also add all middlewares to it
            }
            
    }

    pub fn patch<F, R>(&mut self, path: &str, middlewares: Vec<F>, callback: F) where 
        F: Fn(&mut Request, &mut Response, &mut FlowCtrl, &mut Depot) -> R + Clone + Handler + Send + Sync + 'static,
        R: std::future::Future<Output = ()>{

            let mut hoopRouters = middlewares
                .into_iter()
                .map(|m| Router::new().hoop(m))
                .collect::<Vec<Router>>();
            
            let mut newRouter = Router::with_path(path).patch(callback);
            if let Some(inner_router) = self.clone().take_inner_router(){
                newRouter
                    .push(inner_router) // push the current router to the new one
                    .append(&mut hoopRouters); // also add all middlewares to it
            }
            
    }

    pub fn delete<F, R>(&mut self, path: &str, middlewares: Vec<F>, callback: F) where 
        F: Fn(&mut Request, &mut Response, &mut FlowCtrl, &mut Depot) -> R + Clone + Handler + Send + Sync + 'static,
        R: std::future::Future<Output = ()>{

            let mut hoopRouters = middlewares
                .into_iter()
                .map(|m| Router::new().hoop(m))
                .collect::<Vec<Router>>();
            
            let mut newRouter = Router::with_path(path).delete(callback);
            if let Some(inner_router) = self.clone().take_inner_router(){
                newRouter
                    .push(inner_router) // push the current router to the new one
                    .append(&mut hoopRouters); // also add all middlewares to it
            }
            
    }
    
}




#[tokio::test]
async fn buildServer(){

    let app = Express::new(AppCtx{});

    // wirging  middlewares
    #[handler]
    pub async fn checkUser(
        req: &mut Request, 
        resp: &mut Response, 
        depot: &mut Depot,
        ctrl: &mut FlowCtrl, 
    ){
        // an api middleware 
        // ...
    }

    app.clone().post(
        "/user/get/all", // support wildcards
        vec![(|req, res, ctrl, depot| checkUser)(req, res, ctrl, depot)], // Wrap `checkUser` in a closure
        |req, res, ctrl, depot| async move {
            
            res.render("hello from server");
            ctrl.call_next(req, depot, res).await; // calling the next handler from the router tree


        },
    );


}