mod conf;
use conf::Settings;

use redsumer::*;
use structmap::ToMap;
use structmap_derive::ToMap;
use uuid::Uuid;

#[derive(ToMap, Default)]
pub struct Person {
    id: Uuid,
    name: String,
    last_name: String,
    age: u8,
    still_alive: bool,
}

#[tokio::main]
async fn main() {
    let conf: Settings = Settings::get();

    let producer: RedsumerProducer = match RedsumerProducer::new(
        None,
        conf.get_redis_host(),
        conf.get_redis_port(),
        conf.get_redis_db(),
        conf.get_stream_name(),
    ) {
        Ok(redsumer_producer) => redsumer_producer,
        Err(error) => panic!(
            "Error creating a new RedsumerProducer instance: {:?}",
            error
        ),
    };

    let john = Person {
        id: Uuid::new_v4(),
        name: "John".to_string(),
        last_name: "Doe".to_string(),
        age: 30,
        still_alive: true,
    };

    let _id = match producer.produce(Person::to_stringmap(john)).await {
        Ok(id) => id,
        Err(error) => panic!(
            "Error producing stream message from HashMap: {:?}",
            error.to_string(),
        ),
    };
}
