pub mod structures{
    use rand::Rng;


    #[derive(Clone, Debug, Default)]
    struct Enemy{
        damage_rate: u8
    }

    #[derive(Clone, Debug, Default)]
    struct Player<'s>{
        nickname: &'s str,
        score: u16,
    }

    #[derive(Clone, Debug, Default)]
    struct Col{
        x: u8,
        y: u8
    }

    #[derive(Clone, Debug, Default)]
    struct Row{
        x: u8,
        y: u8
    }

    #[derive(Clone, Debug, Default)]
    struct Board<'b>{
        col: &'b [Col],
        row: &'b [Row]
    }

    #[derive(Clone, Debug, Default)]
    struct Node<T>{
        pub value: T, 
        pub parent: Option<std::sync::Arc<std::rc::Weak<Node<T>>>>,
        pub children: Option<std::sync::Arc<tokio::sync::Mutex<Vec<Node<T>>>>>
    }

    #[derive(Clone, Debug, Default)]
    struct Graph<T>{
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

    pub fn build_game(){

        // every node is has a board instance as its value
        let board = Board::default();
        let mut node = Node::<Board<'_>>::default();
        node.value = board;

    }

}

pub mod functions{
    
}