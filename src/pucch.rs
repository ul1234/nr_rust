use core::{fmt, panic};
use std::cell::RefCell;
use serde_derive::{Serialize, Deserialize};
use crate::err::Error;
use crate::constants::*;
use crate::rrc_pucch::*;

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
    id: PucchResourceSetId,
    pucch_resource_id: Vec<PucchResourceId>,
    max_payload_minus_1: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct PucchResource {
    id: PucchResourceId,
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
    fn from(c: PucchFormatConfigR) -> Self {
        PucchFormatConfig {
            inter_slot_freq_hopping: c.inter_slot_freq_hopping,
            addition_dmrs: c.addition_dmrs,
            max_coderate_x100: match c.max_coderate {
                Some(coderate) => (coderate * 100f32) as u32,
                None => 0,
            },
            num_slots: match c.num_slots {
                Some(slots) => slots,
                None => 1,      // default value
            },
            pi2_bpsk: c.pi2_bpsk,
            simul_harq_csi: c.simul_harq_csi,
        }
    }
}

impl From<PucchConfigR> for PucchConfig {
    fn from(c: PucchConfigR) -> Self {
        let mut pucch_resource_set = Vec::new();
        let default_payload: [u32; 4] = [2, 1706, 1706, 1706];
        for (i, set) in c.pucch_resource_set.unwrap().iter().enumerate() {
            pucch_resource_set.push(
                PucchResourceSet {
                    id: set.id,
                    pucch_resource_id: set.pucch_resource_id.clone(),
                    max_payload_minus_1: match set.max_payload_minus_1 {
                        Some(val) => val,
                        None => default_payload[i],
                }
            });
        }

        let pucch_format1: PucchFormatConfig = c.pucch_format1.into();
        let pucch_format2: PucchFormatConfig = c.pucch_format2.into();
        let pucch_format3: PucchFormatConfig = c.pucch_format3.into();
        let pucch_format4: PucchFormatConfig = c.pucch_format4.into();

        let mut pucch_resource = Vec::new();
        for resource in &c.pucch_resource.unwrap() {
            pucch_resource.push(
                PucchResource {
                    id: resource.id,
                    start_prb: resource.start_prb,
                    intra_slot_freq_hopping: resource.intra_slot_freq_hopping,
                    format: resource.format,
                    max_hold_bits: RefCell::new(0),
                }
            );
        }

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

/*************** impl ***************************/

impl PucchConfig {
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

    // 38.213, 9.2.1
    fn pucch_resource_set_for_uci(&self, o_uci: u32) -> &PucchResourceSet {
        unimplemented!()
    }

    fn pucch_resource_for_uci(&self, o_uci: u32, pucch_resource_idx: u32, dci_first_cce: u32, num_cce: u32) -> &PucchResource {
        unimplemented!()
    }
}

