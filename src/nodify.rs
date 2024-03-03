


// https://drive.google.com/file/d/1Es7Ew8fqkRKGFYfcmFZ8gGJWYOzncA6v/view?usp=sharing



pub mod structures{
    use rand::Rng;

    pub struct NodePerception{
        ltn: i64, // last time this node perception got notified
    }
    
    #[derive(Clone, Debug, Default)]
    pub struct SignalMemory{
        signal_producer: SignalPorducerInfo,
        signal_type: SignalType,
        last_time_sp_seen: i64,
    }

    #[derive(Clone, Debug, Default)]
    pub struct SignalPorducerInfo;

    #[derive(Clone, Debug, Default)]
    pub struct SignalType;

    #[derive(Clone, Debug, Default)]
    pub struct Enemy{
        damage_rate: u8
    }

    #[derive(Clone, Debug, Default)]
    pub struct Player<'s>{
        nickname: &'s str,
        score: u16,
    }

    #[derive(Clone, Debug, Default)]
    pub struct Col{
        x: u8,
        y: u8
    }

    #[derive(Clone, Debug, Default)]
    pub struct Row{
        x: u8,
        y: u8
    }

    #[derive(Clone, Debug, Default)]
    pub struct Board<'b>{
        col: &'b [Col],
        row: &'b [Row]
    }

    #[derive(Clone, Debug, Default)]
    pub struct Node<T>{
        pub value: T, 
        pub parent: Option<std::sync::Arc<std::rc::Weak<Node<T>>>>,
        pub children: Option<std::sync::Arc<tokio::sync::Mutex<Vec<Node<T>>>>>
    }

    #[derive(Clone, Debug, Default)]
    pub struct Graph<T>{
        nodes: Vec<Node<T>>
    }

    impl Enemy{
        fn new() -> Self{
            Self { 
                damage_rate: {
                    let mut rng = rand::thread_rng();
                    let rate = rng.gen_range(0..10);
                    rate
                } 
            }
        }
    }

    

}

pub mod functions{

    pub use super::structures::*;
        
    pub fn build_game(){

        // every node has a board instance as its value
        let mut board = Board::default();
        let mut node = Node::<Board<'_>>::default();
        node.value = board.clone();

    }

    pub fn find_optimised_path(){

    }

}