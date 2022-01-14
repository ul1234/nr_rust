#[macro_use]
mod macros;
mod constants;
mod err;
mod math;
mod optional;
mod pucch;
mod read_config;
mod rrc_pucch;
use pucch::*;
use rrc_pucch::PucchConfigR;

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
        Ok(pucch_config) => {
            println!("{}", pucch_config);
            pucch_config
        }
        Err(e) => {
            println!("{:?}", e);
            return ();
        }
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

    let csi_report = CsiReport {priority: 1, o_csi: 20, o_csi_1: 20, o_csi_2: None};

    let mut channels = vec![
        PucchLogicChannel::new(PucchChannelType::HarqDci, PucchResourceId::new(&pucch_config, 1)),
        PucchLogicChannel::new(PucchChannelType::Sr(SrRequest{positive: true, sr_id: 1}), PucchResourceId::new(&pucch_config, 0)),
        PucchLogicChannel::new(PucchChannelType::Csi(csi_report), PucchResourceId::new(&pucch_config, 2)),
    ];

    pucch_proc(&pucch_config, &mut channels);
}
