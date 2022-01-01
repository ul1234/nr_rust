use core::fmt;
use serde_derive::{Serialize, Deserialize};
use crate::err::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct PucchConfigCommonR {
    index: u32,
    pucch_group_seq_hopping: PucchGroupSeqHopping,
    p0_nominal: i32,
}

#[derive(Debug, Serialize, Deserialize)]
enum PucchGroupSeqHopping {
    Neither,
    GroupHopping(u32),
    SeqHopping(u32),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PucchConfigR {
    pub pucch_resource_set: Option<Vec<PucchResourceSetR>>,
    pub pucch_resource: Option<Vec<PucchResourceR>>,
    pub pucch_format1: PucchFormatConfigR,
    pub pucch_format2: PucchFormatConfigR,
    pub pucch_format3: PucchFormatConfigR,
    pub pucch_format4: PucchFormatConfigR,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PucchResourceSetR {
    pub id: PucchResourceSetId,
    pub pucch_resource_id: Vec<PucchResourceId>,
    pub max_payload_minus_1: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PucchResourceSetId(u32);

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PucchResourceId(u32);

#[derive(Debug, Serialize, Deserialize)]
pub struct PucchResourceR {
    pub id: PucchResourceId,
    pub start_prb: u32,
    pub intra_slot_freq_hopping: IntraSlotFreqHopping,
    pub format: PucchFormat,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum IntraSlotFreqHopping {
    Hopping { second_prb: u32 },
    NoHopping,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum PucchFormat {
    PucchFormat0 { init_cyclic_shift: u32, num_sym: u32, start_sym: u32},
    PucchFormat1 { init_cyclic_shift: u32, num_sym: u32, start_sym: u32, time_occ: u32},
    PucchFormat2 { num_rb: u32, num_sym: u32, start_sym: u32},
    PucchFormat3 { num_rb: u32, num_sym: u32, start_sym: u32},
    PucchFormat4 { num_sym: u32, occ_len: u32, occ_idx: u32, start_sym: u32},
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PucchFormatConfigR {
    pub inter_slot_freq_hopping: bool,
    pub addition_dmrs: bool,
    pub max_coderate: Option<f32>,
    pub num_slots: Option<u32>,
    pub pi2_bpsk: bool,
    pub simul_harq_csi: bool,
}

/********************** impl trait *************************/

impl Default for PucchFormatConfigR {
    fn default() -> Self { 
        Self { inter_slot_freq_hopping: false,
            addition_dmrs: false, 
            max_coderate: Some(0.08),
            num_slots: Some(1), 
            pi2_bpsk: true, 
            simul_harq_csi: true } 
    }
}

impl Default for PucchConfigR {
    fn default() -> Self {
        PucchConfigR {
            pucch_resource_set: Some(
                vec![PucchResourceSetR {
                    id: PucchResourceSetId(0),
                    pucch_resource_id: vec![PucchResourceId(0), PucchResourceId(1)],
                    max_payload_minus_1: None}
                    ]
            ),
            pucch_resource: Some(
                vec![PucchResourceR {
                    id: PucchResourceId(0),
                    start_prb: 0,
                    intra_slot_freq_hopping: IntraSlotFreqHopping::NoHopping,
                    format: PucchFormat::PucchFormat0 { init_cyclic_shift: 0, num_sym: 2, start_sym: 0},
                },
                PucchResourceR {
                    id: PucchResourceId(1),
                    start_prb: 0,
                    intra_slot_freq_hopping: IntraSlotFreqHopping::Hopping {second_prb: 20},
                    format: PucchFormat::PucchFormat1 { init_cyclic_shift: 2, num_sym: 2, start_sym: 3, time_occ: 1},
                }]
            ),
            pucch_format1: PucchFormatConfigR::default(), 
            pucch_format2: PucchFormatConfigR::default(), 
            pucch_format3: PucchFormatConfigR::default(), 
            pucch_format4: PucchFormatConfigR::default(), 
        }
    }
}

impl fmt::Display for PucchConfigR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let json_pretty = serde_json::to_string_pretty(self).expect("cannot serialize pucch_config");
        // println!("{}", pucch_config_json);
        write!(f, "{}", json_pretty)
    }
}
/********************** impl *************************/

impl PucchConfigR {
    fn simul_harq_csi(&self) -> bool {
        self.pucch_format2.simul_harq_csi
    }

    fn max_hold_bits(&self) -> u32 {
        0
    }

    fn valid(&self) -> bool {
        self.pucch_resource_set.is_none()
    }

    pub fn pucch_resource_set(&self, o_uci: u32) -> &PucchResourceSetR {
        self.pucch_resource_set.as_ref().unwrap().iter()
            .find(|s| o_uci <= s.max_payload_minus_1.unwrap_or(1706))
            .unwrap()
    }
}
