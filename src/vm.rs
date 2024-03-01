


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>{


    gvmlib::init_vm();

    Ok(())

}