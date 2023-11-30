

use futures::future::{BoxFuture, FutureExt};

pub const CHARSET: &[u8] = b"0123456789";


// -----------------------------------
// handling a recursive async function
// -----------------------------------
// https://rust-lang.github.io/async-book/07_workarounds/04_recursion.html
// NOTE - Future trait is an object safe trait thus we have to Box it with dyn keyword to have kinda a pointer to the heap where the object is allocated in runtime
// NOTE - a recursive `async fn` will always return a Future object which must be rewritten to return a boxed `dyn Future` to prevent infinite size allocation in runtime from heppaneing some kinda maximum recursion depth exceeded prevention process
// the return type can also be ... -> impl std::future::Future<Output=usize>
// which implements the future trait for the usize output also BoxFuture<'static, usize>
// is a pinned Box under the hood because in order to return a future as a type
// we have to return its pinned pointer since future objects are traits and 
// traits are not sized at compile time thus we have to put them inside the 
// Box or use &dyn to return them as a type and for the future traits we have
// to pin them into the ram in order to be able to solve them later so we must 
// return the pinned Box (Box in here is a smart pointer points to the future)
// or use impl Trait in function return signature. 
//
// async block needs to be pinned into the ram and since they are traits of 
// the Future their pointer will be either Box<dyn Trait> or &dyn Trait, 
// to pin them into the ram to solve them later.
//
// since async blocks are of type Future trait in roder to return them
// as a type their pointer either Box<dyn Trait> or &dyn Trait must be
// pinned into the ram to let us solve them later because rust doesn't 
// have gc and it'll drop the type after it moved into the new scope or
// another type thus for the future objects we must pin them to ram and 
// tell rust hey we're moving this in other scopes but don't drop it because
// we pinned it to the ram to solve it in other scopes, also it must have
// valid lifetime during the the entire lifetime of the app.
//
// BoxFuture<'fut, ()> is Pin<alloc::boxed::Box<dyn Future<Output=()> + Send + Sync + 'fut>>
pub fn async_gen_random_idx(idx: usize) -> BoxFuture<'static, usize>{ // NOTE - pub type BoxFuture<'a, T> = Pin<alloc::boxed::Box<dyn Future<Output = T> + Send + 'a>>
    async move{
        if idx <= CHARSET.len(){
            idx
        } else{
            gen_random_idx(rand::random::<u8>() as usize)
        }
    }.boxed() // wrap the future in a Box, pinning it
}
pub fn ret_boxed_future() -> std::pin::Pin<Box<dyn std::future::Future<Output=()>>>{ // Pin takes a pointer to the type and since traits are dynamic types thir pointer can be either &dyn ... or Box<dyn...>
    // ret future as a pinned box means pinning the pointer of future trait into the ram so they can't move
    Box::pin(async move{ // pinning the box pointer of async block into the ram to solve it later 
        ()
    })
}

// recursive random index generator
pub fn gen_random_idx(idx: usize) -> usize{
    if idx < CHARSET.len(){
        idx
    } else{
        gen_random_idx(rand::random::<u8>() as usize)
    }
}