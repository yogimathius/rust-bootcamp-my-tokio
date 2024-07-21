use rust_bootcamp_my_tokio::my_tokio;

fn main() {
    let my_tokio = my_tokio::MyTokio::new();

    my_tokio.spawn(async {
        println!("Task 1");
    });

    my_tokio.spawn(async {
        println!("Task 2");
    });

    my_tokio.spawn(async {
        println!("Task 3");
    });

    println!("Run my tokio");
    my_tokio.run();
}
