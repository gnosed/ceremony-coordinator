#[macro_use]
extern crate rocket;
use rocket::State;

use phase1_coordinator::{
	authentication::{Dummy, Signature},
	environment::{Development, Environment, Parameters},
	Coordinator,
};

use std::sync::Arc;
use tracing_subscriber;

use tokio::sync::RwLock;

#[get("/")]
fn index() -> String {
	format!("Hello my dear!",)
}

#[get("/update")]
async fn update_coordinator(coordinator: &State<Arc<RwLock<Coordinator>>>) -> () {
	if let Err(error) = coordinator.write().await.update() {
		error!("{}", error);
	}
}

fn instantiate_coordinator(environment: &Environment, signature: Arc<dyn Signature>) -> anyhow::Result<Coordinator> {
	Ok(Coordinator::new(environment.clone(), signature)?)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
	tracing_subscriber::fmt::init();
	// Set the environment.
	let environment: Environment = Development::from(Parameters::TestCustom {
		number_of_chunks: 8,
		power: 12,
		batch_size: 256,
	})
	.into();

	// Instantiate the coordinator.
	let coordinator: Arc<RwLock<Coordinator>> = Arc::new(RwLock::new(
		instantiate_coordinator(&environment, Arc::new(Dummy)).unwrap(),
	));

	let ceremony_coordinator = coordinator.clone();

	// Initialize the coordinator.
	ceremony_coordinator.write().await.initialize().unwrap();

	let rocket = rocket::build()
		.mount("/", routes![index, update_coordinator])
		.manage(ceremony_coordinator)
		.ignite()
		.await?;
	println!("Hello, Rocket: {:?}", rocket);

	let result = rocket.launch().await;
	println!("The server shutdown: {:?}", result);

	result
}
