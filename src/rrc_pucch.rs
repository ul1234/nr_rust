use crate::err::Error;
use core::fmt;
use serde_derive::{Deserialize, Serialize};

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
    pub sr_resource: Option<Vec<SrResourceConfigR>>,
    pub multi_csi_resource: Option<Vec<u32>>, // pucch resource id
    pub dl_data_to_ul_ack: Option<Vec<u32>>,  // K1
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PucchResourceSetR {
    pub pucch_resource_set_id: u32,
    pub pucch_resource_id: Vec<u32>, // pucch resourc id
    pub max_payload_minus_1: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PucchResourceR {
    pub pucch_resource_id: u32,
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
    Format0 { init_cyclic_shift: u32, num_sym: u32, start_sym: u32 },
    Format1 { init_cyclic_shift: u32, num_sym: u32, start_sym: u32, time_occ: u32 },
    Format2 { num_rb: u32, num_sym: u32, start_sym: u32 },
    Format3 { num_rb: u32, num_sym: u32, start_sym: u32 },
    Format4 { num_sym: u32, occ_len: u32, occ_idx: u32, start_sym: u32 },
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SrResourceConfigR {
    pub sr_resource_id: u32,
    pub sr_id: u32,
    pub period_offset: SrPeriodOffset,
    pub pucch_resource_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SrPeriodOffset {
    Sym2,
    Sym6Or7,
    SL1,
    SL2(u32),
    SL4(u32),
    SL5(u32),
    SL8(u32),
    SL10(u32),
    SL16(u32),
    SL20(u32),
    SL40(u32),
    SL80(u32),
    SL160(u32),
    SL320(u32),
    SL640(u32),
}

/********************** impl trait *************************/

impl Default for PucchFormatConfigR {
    fn default() -> Self {
        Self {
            inter_slot_freq_hopping: false,
            addition_dmrs: false,
            max_coderate: Some(0.08),
            num_slots: Some(1),
            pi2_bpsk: true,
            simul_harq_csi: true,
        }
    }
}

impl Default for PucchConfigR {
    fn default() -> Self {
        PucchConfigR {
            pucch_resource_set: Some(vec![PucchResourceSetR {
                pucch_resource_set_id: 0,
                pucch_resource_id: vec![0, 1],
                max_payload_minus_1: None,
            }]),
            pucch_resource: Some(vec![
                PucchResourceR {
                    pucch_resource_id: 0,
                    start_prb: 0,
                    intra_slot_freq_hopping: IntraSlotFreqHopping::NoHopping,
                    format: PucchFormat::Format0 { init_cyclic_shift: 0, num_sym: 2, start_sym: 0 },
                },
                PucchResourceR {
                    pucch_resource_id: 1,
                    start_prb: 0,
                    intra_slot_freq_hopping: IntraSlotFreqHopping::Hopping { second_prb: 20 },
                    format: PucchFormat::Format1 { init_cyclic_shift: 2, num_sym: 2, start_sym: 3, time_occ: 1 },
                },
            ]),
            pucch_format1: PucchFormatConfigR::default(),
            pucch_format2: PucchFormatConfigR::default(),
            pucch_format3: PucchFormatConfigR::default(),
            pucch_format4: PucchFormatConfigR::default(),
            sr_resource: Some(vec![SrResourceConfigR {
                sr_resource_id: 0,
                sr_id: 0,
                period_offset: SrPeriodOffset::SL8(1),
                pucch_resource_id: 1,
            }]),
            multi_csi_resource: Some(vec![3]),
            dl_data_to_ul_ack: Some(vec![2, 3, 4, 5, 6, 7, 8, 9]),
        }
    }
}

impl fmt::Display for PucchConfigR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let json_pretty = serde_json::to_string_pretty(self).expect("cannot serialize pucch_config");
        write!(f, "{}", json_pretty)
    }
}
