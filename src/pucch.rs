use core::{fmt, panic};
use std::cell::RefCell;
use serde_derive::{Serialize, Deserialize};
use crate::err::Error;
use crate::constants::*;
use crate::rrc_pucch::*;
use crate::math::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PucchConfig {
    pucch_resource_set: Vec<PucchResourceSet>,
    pucch_resource: Vec<PucchResource>,
    pucch_format1: PucchFormatConfig,
    pucch_format2: PucchFormatConfig,
    pucch_format3: PucchFormatConfig,
    pucch_format4: PucchFormatConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct PucchResourceSet {
    pucch_resource_set_id: u32,
    pucch_resource_id: Vec<u32>,
    max_payload_minus_1: u32,
    pucch_resource_index: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PucchResource {
    pucch_resource_id: u32,
    start_prb: u32,
    intra_slot_freq_hopping: IntraSlotFreqHopping,
    format: PucchFormat,
    max_hold_bits: RefCell<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PucchFormatConfig {
    inter_slot_freq_hopping: bool,
    addition_dmrs: bool,
    max_coderate_x100: u32,  // * 100
    num_slots: u32,
    pi2_bpsk: bool,
    simul_harq_csi: bool,
}

/*************** impl trait **********************/

impl fmt::Display for PucchConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let json_pretty = serde_json::to_string_pretty(self).expect("cannot serialize pucch_config");
        write!(f, "{}", json_pretty)
    }
}

impl From<PucchFormatConfigR> for PucchFormatConfig{
    fn from(format_config_rrc: PucchFormatConfigR) -> Self {
        fn max_coderate_x100(val: Option<f32>) -> u32 {
            let max_coderate = val.expect("max code rate not config!");
            const MAX_CODERATE_CANDIDATES: &[f32] = &[0.08, 0.15, 0.25, 0.35, 0.45, 0.6, 0.8];
            assert!(MAX_CODERATE_CANDIDATES.contains(&max_coderate), "invalid max code rate {}!", max_coderate);
            (max_coderate * 100f32) as u32
        }

        fn num_slots(val: Option<u32>) -> u32 {
            let num_slots = val.unwrap_or(1);   // default: 1
            const NUM_SLOTS_CANDIDATES: &[u32] = &[1, 2, 4, 8];
            assert!(NUM_SLOTS_CANDIDATES.contains(&num_slots), "invalid num slots {}!", num_slots);
            num_slots
        }

        PucchFormatConfig {
            inter_slot_freq_hopping: format_config_rrc.inter_slot_freq_hopping,
            addition_dmrs: format_config_rrc.addition_dmrs,
            max_coderate_x100: max_coderate_x100(format_config_rrc.max_coderate),
            num_slots: num_slots(format_config_rrc.num_slots),
            pi2_bpsk: format_config_rrc.pi2_bpsk,
            simul_harq_csi: format_config_rrc.simul_harq_csi,
        }
    }
}

impl From<PucchConfigR> for PucchConfig {
    fn from(pucch_rrc: PucchConfigR) -> Self {
        let pucch_resource = pucch_rrc.pucch_resource.unwrap().iter().map(
            |resource| PucchResource {
                pucch_resource_id: resource.pucch_resource_id,
                start_prb: resource.start_prb,
                intra_slot_freq_hopping: resource.intra_slot_freq_hopping,
                format: resource.format,
                max_hold_bits: RefCell::new(0),
            }
        ).collect::<Vec<_>>();

        const DEFAULT_PAYLOAD_SIZE: [u32; 4] = [2, 1706, 1706, 1706];
        let pucch_resource_set = pucch_rrc.pucch_resource_set.unwrap().iter().enumerate().map(
            |(i, set)| PucchResourceSet {
                pucch_resource_set_id: set.pucch_resource_set_id,
                pucch_resource_id: set.pucch_resource_id.clone(),
                max_payload_minus_1: match set.max_payload_minus_1 {
                    Some(val) => val,
                    None => DEFAULT_PAYLOAD_SIZE[i],
                },
                pucch_resource_index: set.pucch_resource_id.iter().map(|id| PucchConfig::pucch_resource_index(&pucch_resource, *id)).collect(),
            }
        ).collect::<Vec<_>>();

        let pucch_format1: PucchFormatConfig = pucch_rrc.pucch_format1.into();
        let pucch_format2: PucchFormatConfig = pucch_rrc.pucch_format2.into();
        let pucch_format3: PucchFormatConfig = pucch_rrc.pucch_format3.into();
        let pucch_format4: PucchFormatConfig = pucch_rrc.pucch_format4.into();

        let pucch_config = PucchConfig {
            pucch_resource_set,
            pucch_resource,
            pucch_format1,
            pucch_format2,
            pucch_format3,
            pucch_format4,
        };

        for pucch_resource in &pucch_config.pucch_resource {
            *pucch_resource.max_hold_bits.borrow_mut() = pucch_config.max_hold_bits(pucch_resource);
        }

        pucch_config
    }
}

/*************** impl config time ***************************/

impl PucchConfig {
    fn pucch_resource_index(pucch_resource: &[PucchResource], pucch_resource_id: u32) -> usize {
        let (index, _resource) = pucch_resource.iter().enumerate()
            .find(|(_i, resource)| resource.pucch_resource_id == pucch_resource_id)
            .unwrap_or_else(|| panic!("pucch resource id {} not found!", pucch_resource_id));
        index
    }

    fn pucch_format_config(&self, pucch_resource: &PucchResource) -> Option<&PucchFormatConfig> {
        match pucch_resource.format {
            PucchFormat::PucchFormat1{..} => Some(&self.pucch_format1),
            PucchFormat::PucchFormat2{..} => Some(&self.pucch_format2),
            PucchFormat::PucchFormat3{..} => Some(&self.pucch_format3),
            PucchFormat::PucchFormat4{..} => Some(&self.pucch_format4),
            _ => panic!("impossible to be here!"),
        }
    }

    fn pucch_num_dmrs_sym(&self, pucch_resource: &PucchResource) -> u32 {
        self.pucch_dmrs_pos(pucch_resource).len() as u32
    }

    // 38.211, Table 6.4.1.3.3.2-1, DMRS for PUCCH format 3 and 4
    fn pucch_dmrs_pos(&self, pucch_resource: &PucchResource) -> &[u32] {
        let num_sym = match &pucch_resource.format {
            PucchFormat::PucchFormat3{num_rb: _, num_sym, ..} => *num_sym,
            PucchFormat::PucchFormat4{num_sym, ..} => *num_sym,
            _ => panic!("impossible to be here!"),
        };

        if num_sym == 4 {
            match pucch_resource.intra_slot_freq_hopping {
                IntraSlotFreqHopping::Hopping{..} => &[0, 2],
                IntraSlotFreqHopping::NoHopping => &[1],
            }
        }
        else {
            const START_SYM: u32 = 5;
            const NUM_TABLE: usize = (NUM_SYM_PER_SLOT - START_SYM + 1) as usize;
            let pucch_format_config = self.pucch_format_config(pucch_resource).unwrap();
            let index: usize = (num_sym - START_SYM) as usize;
            if pucch_format_config.addition_dmrs {
                const PUCCH_DMRS_POS_TABLE: [&'static [u32]; NUM_TABLE] = [&[0, 3], &[1, 4], &[1, 4], &[1, 5], &[1, 6], &[1, 3, 6, 8], &[1, 3, 6, 9], &[1, 4, 7, 10], &[1, 4, 7, 11], &[1, 5, 8, 12]];
                PUCCH_DMRS_POS_TABLE[index]
            }
            else {
                const PUCCH_DMRS_POS_TABLE: [&'static [u32]; NUM_TABLE] = [&[0, 3], &[1, 4], &[1, 4], &[1, 5], &[1, 6], &[2, 7], &[2, 7], &[2, 8], &[2, 9], &[3, 10]];
                PUCCH_DMRS_POS_TABLE[index]
            }
        }
    }

    // 38.213, 9.2.5.2
    fn max_hold_bits(&self, pucch_resource: &PucchResource) -> u32 {
        match &pucch_resource.format {
            PucchFormat::PucchFormat0{..} => 2,
            PucchFormat::PucchFormat1{..} => 2,
            _ => {
                let pucch_format_config = self.pucch_format_config(pucch_resource).unwrap();
                let num_bits = match &pucch_resource.format {
                    PucchFormat::PucchFormat2{num_rb, num_sym, ..} => {
                        let data_sc = NUM_SC_PER_RB - 4;
                        let qm = QPSK_BITS;
                        pucch_format_config.max_coderate_x100 * (*num_rb) * data_sc * (*num_sym) * qm
                    },
                    PucchFormat::PucchFormat3{num_rb, num_sym, ..} => {
                        let data_sc = NUM_SC_PER_RB;
                        let num_data_sym = *num_sym - self.pucch_num_dmrs_sym(pucch_resource);
                        let qm = if pucch_format_config.pi2_bpsk {BPSK_BITS} else {QPSK_BITS};
                        pucch_format_config.max_coderate_x100 * (*num_rb) * data_sc * num_data_sym * qm
                    },
                    PucchFormat::PucchFormat4{num_sym, occ_len,..} => {
                        let data_sc = NUM_SC_PER_RB / *occ_len;
                        let num_data_sym = *num_sym - self.pucch_num_dmrs_sym(pucch_resource);
                        let qm = if pucch_format_config.pi2_bpsk {BPSK_BITS} else {QPSK_BITS};
                        pucch_format_config.max_coderate_x100 * data_sc * num_data_sym * qm
                    },
                    _ => panic!("impossible to be here!"),
                };

                num_bits / 100
            }
        }
    }
}

/******************** impl runtime **********************/
impl PucchConfig {
    // 38.213, 9.2.1
    fn pucch_resource_set_for_uci(&self, o_uci: u32) -> &PucchResourceSet {
        self.pucch_resource_set.iter().find(|set| o_uci <= set.max_payload_minus_1).unwrap()
    }

    fn pucch_resource_for_uci(&self, o_uci: u32, pucch_resource_indicator: u32, dci_first_cce: u32, num_cce: u32) -> &PucchResource {
        let pucch_resource_set = self.pucch_resource_set_for_uci(o_uci);
        let pucch_resource_index = match pucch_resource_set.pucch_resource_set_id {
            0 => {
                // 38.213, 9.2.3
                let num_pucch_resource = pucch_resource_set.pucch_resource_id.len() as u32;
                if num_pucch_resource > 8 {
                    let pucch_resource_index = floor(dci_first_cce * ceil(num_pucch_resource, 8), num_cce) + 
                        pucch_resource_indicator *  ceil(num_pucch_resource, 8);
                    let num_pucch_resource_mod_8 = num_pucch_resource % 8;
                    if pucch_resource_indicator < num_pucch_resource_mod_8 {
                        pucch_resource_index
                    }
                    else {
                        pucch_resource_index + num_pucch_resource_mod_8
                    }
                }
                else {
                    pucch_resource_indicator
                }
            },
            1..=3 => {
                pucch_resource_indicator
            },
            _ => panic!("impossible to be here!"),
        };

        let pucch_resource_index = pucch_resource_set.pucch_resource_index[pucch_resource_index as usize];
        &self.pucch_resource[pucch_resource_index]
    }
}

