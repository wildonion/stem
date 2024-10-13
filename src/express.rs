

use crate::*;
use cors::Cors;
use handler::HoopedHandler;
use prelude::{max_concurrency, Compression, CompressionLevel};
use salvo::*;
use salvo::http::Method;
use test::ResponseExt;
use std::{os::unix::thread, pin::pin, process::Output, sync::{mpsc, Arc}};




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
    pub fn post<C, M, R>(&mut self, path: &str, middlewares: &[M], callback: C) where 
        M: Send + Sync + Clone + 'static + Handler, // structure middelware
        C: Send + Sync + 'static + Handler, // structure middelware
        R: std::future::Future<Output = ()>{

            let mut hoopRouters = middlewares.to_vec()
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
    
}




#[tokio::test]
async fn buildServer(){

    // api or middleware handlers
    #[handler]
    pub async fn ensureAdminAccess(){}
    #[handler]
    pub async fn ensureUserAccess(){}


    // since each handler method would convert into a structure who impls the Handler
    // trait we can box them as an instance of the struct and collect them through a
    // single interface using the dynamic dispatch and dependency injection approach.
    let handlers: Vec<Box<dyn Handler>> = vec![Box::new(ensureAdminAccess), Box::new(ensureUserAccess)];
 
    post!(
        "/user",
        (ensureAdminAccess, ensureUserAccess),
        (req, res, next, depot) => async move { // if you don't have a variable just define it as ident which is a name for that
            // Handler logic
            res.render("hello");
            println!("Handling request for /user");
        }
    );

}

#[macro_export]
macro_rules! post {
    /* 
        due to the static dispatch behaviour
        the `mismatched types
            expected fn item `fn() -> impl std::future::Future<Output = ()> {ensureAdminAccess}`
            found fn item `fn() -> impl std::future::Future<Output = ()> {ensureUserAccess}`
            distinct uses of `impl Trait` result in different opaque types` error
        is caused by the fact that Rust's impl Trait 
        creates an opaque type at compile time, meaning that every use of impl Trait in 
        a different function signature produces a distinct type, even if the returned type 
        is conceptually the same. When you try to collect these middlewares into a Vec, 
        Rust sees each middleware function as returning a distinct type that's why all the
        types inside the vector are not the same since async fn metho(){} uses impl Trait 
        in the background, To solve this, we need to avoid trying to store the impl Trait 
        directly in a Vec. Instead, we can use a trait object (Box<dyn Handler>) to store the 
        middlewares in a homogeneous container (like a Vec).
    */
    (
        $path:expr, ($($mid:ident),*), 
        ($req:ident, $res:ident, 
         $next:ident, $depot:ident) => async move $handler:block) => 
    {
        {
            /* -------------------
                every impl Trait or static dispatch would generate distinct type at
                compile time hence we can't have a vector of impl Trait cause they
                are different event they return a same result we should use dynamic 
                dispatch approach to access multiple types through a single interface.
                in case of trait objects we can box an instance of a type who impls 
                the object safe trait we can collect them into a vector this approach 
                would be used in dynamic dispatch and dep injection.
                collecting the middlewares into a vector since we can't use impl Trait 
                in a vector due to its static type behaviour, we can collect a vector 
                of object safe traits which are injected dependencies. #[handler]
                proc macro in salvo on top of functions convert them into structure
                and impl Handler for the structure which allows us to collect a vector
                of object safe traits by boxing the instance of the struct who impls 
                the Handler trait, this is a dynamic dispatch and since the actual 
                type will be specefied at runtime it's ok to have this instead of 
                impl Trait which is static dispatch.
            */
            let middlewares: Vec<Box<dyn Handler>> = vec![$(Box::new($mid)),*];

            #[handler]
            async fn api(
                $req: &mut Request, 
                $res: &mut Response, 
                $depot: &mut Depot, 
                $next: &mut FlowCtrl
            ){
                $handler
            }

        }
    };
}
