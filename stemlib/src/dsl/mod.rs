
// -----------------------------------
/* Neuron Actor DSL using Macros

    TODOs: dsl.rs, select!{}, publishToRmq() and consumeFromRmq() methods, 
    TODOs: stem.spec, express framework, publish stemlib to crate
    tools: 
        GPT tryout section in stem.spec, 
        desktop books for neuroscience and information theory
    features:
        local talking with jobq chan eventloop receiver of mailbox of actor message sending logic
        encrypted through crypter crate of remote talking through p2p and rpc rmq req-rep queue:
            p2p docs https://docs.ipfs.tech/concepts/libp2p/ , https://docs.libp2p.io/
            p2p based like adnl file sharing, vpn like tor, ton and v2ray, firewall, gateway, loadbalancer, ingress listener like ngrok, proxy and dns over neuron actor
            p2p based like adnl onchain broker stock engine (find peers which are behind nat over wan)
    main concepts:
        dyn dispatch and dep injection with Arc::new(TypeImplsTrait{}) Box::new(TypeImplsTrait{}) Arc::pin(async move{}), Box::pin(async move{})
        stat dyn dispatch, dep injecton and binding to trait, Box::pin(async move{}), Arc::pin(async move{}), Arc<Fn() -> R> where R: Future
        Err(CstmError::new()) || ? on object of type CstmError who impls Error, From, Display traits || Err(Box::new(CstmError::new()))
        use Box::pin(async move{}) or Arc::pin(async move{})  to return async future object in none async context that you can't await on future objects
        make everything cloneable and break the cycle of self ref types using Arc and store on the heap using Box 
        mutex, channel, spawn, select, trait closure for poly, dep inj, dyn and stat dispatch,
        future objects in form of dyn dispatch with Box::pin(async move{}) or a job in closure return type with Arc::new(||async move{})
        raft,dag,mdp,adjmat,merkletree,shard,replica,p2p::wrtc,udp,quic,tcp,ws,noise,gossipsub,kdht,swarm,lightthread
        atomic,chan,arc,mutex,select,spawn,eventloop,CronScheduler,send,sync,static, static lazy arc mutex app ctx + send + sync
        thread_local, actor id or address, std::alloc, jemalloc, GlobalAlloc, bumpalo and r3bl_rs_utils arena as a global allocator
        zero copy, null pointer optimiser, unique storage key, dsl and feature based for stem
        atomic transaction: ALL OR NONE with atomic syncing using static lazy Arc Mutex & Channels 
        default type param, default const in struct, let ONION = const{}, Arc<Mutex< to mutate arced value vs Rc<RefCell< to mutate Rced value
        interfaces and traits for poly, stat dyn dispatch, access types through a single interface, Any trait, dep injection
        if you don't care about the result of the io task don't await on the spawn otherwise use static lazy arc mutex or chans and let the task gets executed in the background thread
        spawn(async move{handleMsg().await}) in the background light thread (none blocking) without awaiting: thread per each task
        Box::pin(async move{}), Arc::pin(async move{}) and Arc<Fn() -> R> where R: Future + Send + Sync
        eventloop with spawn(async move{loop{select!{}}}) and spawn(async move{while let Some(job) = rx.recv().await{}}) inside the actor.rs of the stem 
        CronScheduler(time, ctx, redis pubsub exp key), select!{} awaiting, arc, mutex, timeout, Box::pin(async{}), Arc::pin(async move{}), condvar, jobq chan send recv

        proc ones can only be used on top of functions and impl blocks to extend 
        the function body and add some vars into it at compile time:
            use proc macro with attrs to handle automatic task spawning and jobq chan creations
            use decl macro to build dsl

        struct ProcessStruct;

        // also handle multiple passing params to function using macros with omitting the useless ones
        // do this:    
        #[processExt]
        impl ProcessStruct{
            async fn start(&self){}
            async fn stop(&self){}
        }
        // instead of:
        impl processExt for ProcessStruct{} 


        // when a function is annotated with distribute 
        // means it can distribute itself among networks
        #[distribute]
        pub async fn injectShellcodeLogic(){
        }

         ---------------- MACRO PATTERNS -----------------

        rust types can be fallen into one the following categories

        item      ➔ an Item | an item, like a function, struct, module, etc.
        block     ➔ a BlockExpression | a block (i.e. a block of statements and/or an expression, surrounded by braces)
        stmt      ➔ a Statement without the trailing semicolon (except for item statements that require semicolons)
        pat_param ➔ a PatternNoTopAlt
        pat       ➔ at least any PatternNoTopAlt, and possibly more depending on edition
        expr      ➔ an Expression
        ty        ➔ a Type
        ident     ➔ an IDENTIFIER_OR_KEYWORD or RAW_IDENTIFIER
        path      ➔ a TypePath style path | a path (e.g. foo, ::std::mem::replace, transmute::<_, int>, …)
        tt        ➔ a TokenTree (a single token or tokens in matching delimiters (), [], or {})
        meta      ➔ an Attr, the contents of an attribute | a meta item; the things that go inside #[...] and #![...] attributes
        lifetime  ➔ a LIFETIME_TOKEN
        vis       ➔ a possibly empty Visibility qualifier
        literal   ➔ matches -?LiteralExpression
*/


use crate::*;


// a neuron is an actor, an isolated state talks locally and remotely through 
// eventloop jobq channel and rpc rmq + p2p
#[macro_export]
macro_rules! neuron {
    () => {
        {
            
        }
    };
}

// stream is tool helps to start streaming over a neuron actor 
// either locally or remotely  
#[macro_export]
macro_rules! stream {
    () => {
        {

        }
    };
}

// layer contains one or more neurons inside itself, multiple 
// layers form an onion brain
#[macro_export]
macro_rules! layer {
    () => {
        {

        }
    };
}

#[macro_export]
macro_rules! o_O {
    
        (
            $x:expr, [$(t:tt | $em:ty),*]
        ) => {
            &[ $($( $x + $y ), *), * ]
        };

        (
            $(
                $x:expr; [ $( $y:expr ), * ]
            ); 
        *) /* multiple of this pattern */ => {
        
        }
}
//////
/// let a: &[i32] = o_O![10; [1, 2, 3]; 20; [4, 5, 6]];
//////

#[macro_export]
macro_rules! list {
    ($id1:ident | $id2:ident <- [$start:expr; $end:expr], $cond:expr) => { //// the match pattern can be any syntax :) - only ident can be followed by some symbols and words like <-, |, @ and etc
        { //.... code block to return vec since if we want to use let statements we must be inside {} block
            let mut vec = Vec::new();
            for num in $start..$end + 1{
                if $cond(num){
                    vec.push(num);
                }
            }
            vec
        } //....
    };
}
//////
/// let even = |x: i32| x%2 == 0;
/// let odd = |x: i32| x%2 != 0;
/// let evens = list![x | x <- [1; 10], even];
//////

#[macro_export]
macro_rules! dict {
    ($($key:expr => $val:expr)*) => { //// if this pattern matches the input the following code will be executed - * means we can pass more than one key => value statement
        { //.... code block to return vec since if we want to use let statements we must be inside {} block
            use std::collections::HashMap;
            let mut map = HashMap::new();
            $(
                map.insert($key, $value);
            )* //// * means we're inserting multiple key => value statement inside the map 
            map
        } //....
    };
}
//////
/// let d = dict!{"wildonion" => 1, "another_wildonion" => 2, "array": vec![1,3,4235,], "age": 24};
//////

#[macro_export]
macro_rules! exam {
    ($l:expr; and $r:expr) => { //// logical and match 
        $crate::macros::even(); //// calling even() function which is inside the macros module
        println!("{}", $l && $r);
    };

    ($l:expr; or $r:expr) => { //// logical or match 
        println!("{}", $l || $r);
    };
}
//////
/// exam!(1 == 2; and 3 == 2+1)
/// exam!(1 == 2; or 3 == 2+1)
//////


#[macro_export]
macro_rules! cmd {
    ($iden:ident, $ty: tt) => {
        pub struct $iden(pub $ty);
        impl Default for $iden{
            fn default() -> Self{
                todo!()
            }
        }  
    };

    ($func_name:ident) => {
        fn $func_name(){
            println!("you've just called {:?}()", stringify!($func_name));
        }
    }
}
//////
/// cmd!{bindgen, id} //// bindgen is the name of the struct and id is the name of the field
//////


#[macro_export]
macro_rules! query { // NOTE - this is a macro with multiple syntax support and if any pattern matches with the caller pattern, then the code block of that pattern will be emitted
    
    ( $value_0:expr, $value_1:expr, $value_2:expr ) => { //// passing multiple object syntax
        // ...
    };

    ( $($name:expr => $value:expr)* ) => { //// passing multiple key => value syntax 
        // ...

    };

}

#[macro_export]
macro_rules! dynamic_methods {
    ($builder:ident, $($field:ident: $field_type:ty),*) => {
        impl $builder {
            $(
                pub fn $field(mut self, $field: $field_type) -> Self {
                    self.$field = Some($field);
                    self
                }
            )*
        }
    };
}
//////
/// dynamic_methods!{StructName, id: None, name: None, age: i32}
//////

#[macro_export]
macro_rules! log {
    ($arg:tt) => { //// passing single String message, tt is type
        $crate::env::log($arg.as_bytes()) //// log function only accepts utf8 bytes
    };
    ($($arg:tt)*) => { //// passing multiple String messages 
        $crate::env::log(format!($($arg)*).as_bytes()) //// log function only accepts utf8 bytes
    };
}


#[macro_export]
macro_rules! impl_ecq_engine_constructor {
    ($( $new:ident: [ $( $pos:expr ),* ] anchored at $anchor:expr; )*) => { //// the match pattern can be any syntax :) - only ident can be followed by some symbols and words like <-, |, @ and etc 
        $(
            pub fn $new() -> Self{
                Self{
                    positions: [$( $pos ),*].into_iter().collect(),
                    anchor: $anchor,
                }
            }
        )* //// * means defining function for every new Pos
    };
}

// #[derive(Debug, Clone)]
// pub struct Shape{
//     typ: &'static str,
//     positions: HashSet<Pos>,
//     anchor: Pos,
// }


// #[derive(Debug, Clone, Copy)]
// pub struct Pos(pub i32, pub i32);



// impl Shape {
//     impl_ecq_engine_constructor! {
//       new_i "🟦": [Pos(0, 0), Pos(1, 0), Pos(2, 0), Pos(3, 0)] @ Pos(1, 0);
//       new_o "🟨": [Pos(0, 0), Pos(1, 0), Pos(0, 1), Pos(1, 1)] @ Pos(0, 0);
//       new_t "🟫": [Pos(0, 0), Pos(1, 0), Pos(2, 0), Pos(1, 1)] @ Pos(1, 0);
//       new_j "🟪": [Pos(0, 0), Pos(0, 1), Pos(0, 2), Pos(-1, 2)] @ Pos(0, 1);
//       new_l "🟧": [Pos(0, 0), Pos(0, 1), Pos(0, 2), Pos(1, 2)] @ Pos(0, 1);
//       new_s "🟩": [Pos(0, 0), Pos(1, 0), Pos(0, 1), Pos(-1, 1)] @ Pos(0, 0);
//       new_z "🟥": [Pos(0, 0), Pos(-1, 0), Pos(0, 1), Pos(1, 1)] @ Pos(0, 0);
//     }
// }

#[macro_export]
macro_rules! iterator{
    ($ty:ty, $ident:ident; $($state_ident:ident: $state_ty:ty),*; $next:expr) => (
        struct $ident {
            $($state_ident: $state_ty), *
        }

        impl Iterator for $ident {
            type Item = $ty;

            fn next(&mut self) -> Option<$ty> {
                $next(self)
            }
        }
    );
}
//////
// iterator!(i32, TestIterator; index: i32; |me: &mut TestIterator| {
//     let value = Some(me.index);
//     me.index += 1;
//     value
// });
//////


macro_rules! pat {
    ($i:ident) => (Some($i))
}

// if let pat!(x) = Some(1) {
//     assert_eq!(x, 1);
// }

macro_rules! Tuple {
    { $A:ty, $B:ty } => { ($A, $B) };
}

type N2 = Tuple!(i32, i32);

macro_rules! const_maker {
    ($t:ty, $v:tt) => { const CONST: $t = $v; };
}
trait T {
    const_maker!{i32, 7}
}

macro_rules! example {
    () => { println!("Macro call in a macro!"); };
}

#[macro_export]
macro_rules! contract {

    /*

        contract!{

            NftContract, //// name of the contract
            "wildonion.near", //// the contract owner
            /////////////////////
            //// contract fields
            /////////////////////
            [
                contract_owner: AccountId, 
                deposit_by_owner: HashMap<AccountId, near_sdk::json_types::U128>, 
                contract_balance: near_sdk::json_types::U128
            ]; //// fields
            /////////////////////
            //// contract methods
            /////////////////////
            [ 
                "init" => [ //// array of init methods
                    pub fn init_contract(){
            
                    }
                ],
                "private" => [ //// array of private methods
                    pub fn get_all_deposits(){

                    }
                ],
                "payable" => [ //// array of payable methods
                    pub fn deposit(){
            
                    }
                ],
                "external" => [ //// array of external methods
                    fn get_address_bytes(){

                    }
                ]
            ]

        }

    */

    // event!{
    //     name: "list_owner",
    //     log: [NewOwner, AddDeposit],

    //     // event methods

    //     fn add_owner(){

    //     } 

    //     fn add_deposit(){
            
    //     }
    // }

    // emit!{
    //     event_name
    // }

    (
     $name:ident, $signer:expr, //// ident can be used to pass struct
     [$($fields:ident: $type:ty),*]; 
     [$($method_type:expr => [$($method:item),*]),* ]
    ) 
     
     => {
            #[near_bindgen]
            #[derive(serde::Deserialize, serde::Serialize)]
            pub struct $name{
                $($fields: $type),*
            }

            impl $name{
                        
                // https://stackoverflow.com/questions/64790850/how-do-i-write-a-macro-that-returns-the-implemented-method-of-a-struct-based-on
                // implement methods here 
                // ...
            }
    }
}

#[macro_export]
macro_rules! function {
    ($name:ident, [$($param:ident: $type:ty),*]) => {
        {   
            // since macros extend ast at compile time, it's not possible 
            // to return a function with an empty body to fill up the body
            // later.
            fn $name($($param:$type),*){
                
                $( // iterate through each parameter and include them in the function body

                    println!("{}: {:?}", stringify!($param), $param);
                )*

            }

            $name
        }
    };
}
// #[derive(Clone, Debug)]
// pub struct ExecuteApi;
// let func = function!(
//     set_vals, // function name
//     [msg: ExecuteApi, name: String] // params
// );
// let res = func(msg, String::from(""));

/*
    we can define as many as response object since once the scope
    or method or the match arm gets executed the lifetime of the 
    response object will be dropped from the ram due to the fact 
    that rust doesn't have gc :) 
*/
// #[derive(Serialize, Deserialize, Debug)]
// pub struct Response<'m, T>{
//     pub data: Option<T>,
//     pub message: &'m str, // &str are a slice of String thus they're behind a pointer and every pointer needs a valid lifetime which is 'm in here 
//     pub status: u16,
//     pub is_error: bool
// }
// #[macro_export]
// macro_rules! resp {
//     (   
//         $data_type:ty,
//         $data:expr,
//         $msg:expr,
//         $code:expr,
//         $cookie:expr,
//     ) => {

//         {
//             use actix_web::HttpResponse;
//             use crate::helpers::misc::Response;
            
//             let code = $code.as_u16();
//             let mut res = HttpResponse::build($code);
            
//             let response_data = Response::<$data_type>{
//                 data: Some($data),
//                 message: $msg,
//                 status: code,
//                 is_error: if code == 200 || code == 201 || code == 302{
//                     false
//                 } else{
//                     true
//                 }
//             };
            
//             let resp = if let Some(cookie) = $cookie{
//                 res
//                     .cookie(cookie.clone())
//                     .append_header(("cookie", cookie.value()))
//                     .json(
//                         response_data
//                     )
//             } else{
//                 res
//                     .json(
//                         response_data
//                     )
//             }; 

//             return Ok(resp);
//         }
//     }
// }

//////
// resp!{
//     &[u8], // the data type
//     &[], // response data
//     ACCESS_DENIED, // response message
//     StatusCode::FORBIDDEN, // status code
//     None::<Cookie<'_>>, // cookie
// }
//////