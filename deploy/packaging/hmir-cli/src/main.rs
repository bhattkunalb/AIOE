mod commands;

#[tokio::main]
async fn main() {
    println!("hmir v1.0.0 CLI successfully triggered! Executing status commands across arrays...");
    let recommender = commands::suggest::ModelRecommender::new();
    recommender.suggest("latency");
}
