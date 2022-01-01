mod rrc_pucch;
mod pucch;
mod err;
mod read_config;
mod optional;
use rrc_pucch::PucchConfigR;
use pucch::PucchConfig;

use crate::read_config::load_config;

fn main() {
    println!("Hello, world!");

    // let pucch_config = PucchConfigR::default();
    // println!("{:?}", pucch_config);

    // let pucch_config_json = serde_json::to_string(&pucch_config).expect("cannot serialize pucch_config");
    // let pucch_config_json_pretty = serde_json::to_string_pretty(&pucch_config).expect("cannot serialize pucch_config");

    // println!("{}", pucch_config_json);
    // println!("{}", pucch_config_json_pretty);

    let rrc_pucch_config = match load_config::<PucchConfigR>("input/pucch_config.json") {
        Ok(pucch_config) => {println!("{}", pucch_config); pucch_config},
        Err(e) => { println!("{:?}", e); return () },
    };

    let pucch_config: PucchConfig = rrc_pucch_config.into();
    println!("{}", pucch_config);

    // pucch_config.init_optional();
    // pucch_config.set_default_value();
    // if let Err(e) = pucch_config.check() {
    //     print!("Error: {:?}", e);
    // }

    // let pucch_resource_set = pucch_config.pucch_resource_set(11);

    // println!("{:?}", pucch_resource_set);
}
