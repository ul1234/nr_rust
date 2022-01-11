use crate::constants::*;
use crate::math::*;
use crate::rrc_pucch::*;
use core::{fmt, panic};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PucchConfig {
    pucch_resource_set: Vec<PucchResourceSet>,
    pucch_resource: Vec<PucchResource>,
    pucch_formats: PucchFormatsConfig,
    sr_resource: Option<Vec<SrResourceConfig>>,
    multi_csi_resource: Option<Vec<PucchResourceId>>,
    dl_data_to_ul_ack: Option<Vec<u32>>, // K1
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PucchFormatsConfig {
    pucch_format1: PucchFormatConfig,
    pucch_format2: PucchFormatConfig,
    pucch_format3: PucchFormatConfig,
    pucch_format4: PucchFormatConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct PucchResourceSet {
    pucch_resource_set_id: u32,
    pucch_resource_id: Vec<PucchResourceId>,
    max_payload_minus_1: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PucchResourceId {
    id: u32,
    idx: usize, // to optimize the retrieve of pucch resource
}

#[derive(Debug, Serialize, Deserialize)]
struct PucchResource {
    pucch_resource_id: u32,
    start_prb: u32,
    intra_slot_freq_hopping: IntraSlotFreqHopping,
    format: PucchFormat,
    format_type: PucchFormatType,
    max_hold_bits: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct PucchFormatConfig {
    inter_slot_freq_hopping: bool,
    addition_dmrs: bool,
    max_coderate_x100: u32, // * 100
    num_slots: u32,
    pi2_bpsk: bool,
    simul_harq_csi: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SrResourceConfig {
    sr_resource_id: u32,
    sr_id: u32,
    period: u32, // slot
    offset: u32,
    pucch_resource_id: PucchResourceId,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum PucchFormatType {
    ShortPucch,
    LongPucch,
}

/*************** impl trait **********************/

impl fmt::Display for PucchConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let json_pretty = serde_json::to_string_pretty(self).expect("cannot serialize pucch_config");
        write!(f, "{}", json_pretty)
    }
}

impl From<PucchFormatConfigR> for PucchFormatConfig {
    fn from(format_config_rrc: PucchFormatConfigR) -> Self {
        fn max_coderate_x100(val: Option<f32>) -> u32 {
            let max_coderate = val.expect("max code rate not config!");
            const MAX_CODERATE_CANDIDATES: &[f32] = &[0.08, 0.15, 0.25, 0.35, 0.45, 0.6, 0.8];
            assert!(MAX_CODERATE_CANDIDATES.contains(&max_coderate), "invalid max code rate {}!", max_coderate);
            (max_coderate * 100f32) as u32
        }

        fn num_slots(val: Option<u32>) -> u32 {
            let num_slots = val.unwrap_or(1); // default: 1
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
        let pucch_formats = PucchFormatsConfig {
            pucch_format1: pucch_rrc.pucch_format1.into(),
            pucch_format2: pucch_rrc.pucch_format2.into(),
            pucch_format3: pucch_rrc.pucch_format3.into(),
            pucch_format4: pucch_rrc.pucch_format4.into(),
        };

        let pucch_resource = pucch_rrc
            .pucch_resource
            .unwrap()
            .iter()
            .map(|resource| {
                let temp_pucch_resource = PucchResource {
                    pucch_resource_id: resource.pucch_resource_id,
                    start_prb: resource.start_prb,
                    intra_slot_freq_hopping: resource.intra_slot_freq_hopping,
                    format: resource.format,
                    format_type: PucchResource::pucch_format_type(&resource.format),
                    max_hold_bits: 0,
                };
                PucchResource { max_hold_bits: temp_pucch_resource.max_hold_bits(&pucch_formats), ..temp_pucch_resource }
            })
            .collect::<Vec<_>>();

        const DEFAULT_PAYLOAD_SIZE: [u32; 4] = [2, 1706, 1706, 1706];
        let pucch_resource_set = pucch_rrc
            .pucch_resource_set
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, set)| PucchResourceSet {
                pucch_resource_set_id: set.pucch_resource_set_id,
                pucch_resource_id: set
                    .pucch_resource_id
                    .iter()
                    .map(|&id| PucchResourceId::from_pucch_resource_id(&pucch_resource, id))
                    .collect::<Vec<_>>(),
                max_payload_minus_1: match set.max_payload_minus_1 {
                    Some(val) => val,
                    None => DEFAULT_PAYLOAD_SIZE[i],
                },
            })
            .collect::<Vec<_>>();

        let multi_csi_resource = PucchConfig::from_multi_csi_resource(&pucch_resource, &pucch_rrc.multi_csi_resource);

        PucchConfig {
            pucch_resource_set,
            pucch_resource,
            pucch_formats,
            sr_resource: None,
            multi_csi_resource,
            dl_data_to_ul_ack: None,
        }
    }
}

/*************** impl config time ***************************/
impl PucchResourceId {
    fn from_pucch_resource_id(pucch_resource: &[PucchResource], resource_id: u32) -> PucchResourceId {
        PucchResourceId { id: resource_id, idx: PucchResourceId::pucch_resource_idx_cfg(pucch_resource, resource_id) }
    }

    fn pucch_resource_idx_cfg(pucch_resource: &[PucchResource], resource_id: u32) -> usize {
        pucch_resource
            .iter()
            .position(|resource| resource.pucch_resource_id == resource_id)
            .unwrap_or_else(|| panic!("pucch resource id {} not found!", resource_id))
    }

    fn pucch_resource<'a>(&'a self, pucch_config: &'a PucchConfig) -> &'a PucchResource {
        &pucch_config.pucch_resource[self.idx]
    }
}

impl PucchConfig {
    fn from_multi_csi_resource(
        pucch_resource: &[PucchResource],
        multi_csi_resource: &Option<Vec<u32>>,
    ) -> Option<Vec<PucchResourceId>> {
        match multi_csi_resource {
            Some(resources) => {
                assert!(resources.len() <= 2, "Invalid multi_csi_resource {:?}!", resources);
                let mut pucch_resource_id = resources
                    .iter()
                    .map(|&id| PucchResourceId::from_pucch_resource_id(&pucch_resource, id))
                    .collect::<Vec<_>>();
                // sort the pucch resource from small capacity to large capacity
                pucch_resource_id.sort_by_key(|resource_id| pucch_resource[resource_id.idx].max_hold_bits);
                Some(pucch_resource_id)
            }
            None => None,
        }
    }
}

impl PucchResource {
    fn pucch_format_config<'a>(&'a self, pucch_formats: &'a PucchFormatsConfig) -> &'a PucchFormatConfig {
        match self.format {
            PucchFormat::Format1 { .. } => &pucch_formats.pucch_format1,
            PucchFormat::Format2 { .. } => &pucch_formats.pucch_format2,
            PucchFormat::Format3 { .. } => &pucch_formats.pucch_format3,
            PucchFormat::Format4 { .. } => &pucch_formats.pucch_format4,
            _ => panic!("impossible to be here!"),
        }
    }

    fn pucch_num_dmrs_sym(&self, pucch_formats: &PucchFormatsConfig) -> u32 {
        self.pucch_dmrs_pos(pucch_formats).len() as u32
    }

    // 38.211, Table 6.4.1.3.3.2-1, DMRS for PUCCH format 3 and 4
    fn pucch_dmrs_pos(&self, pucch_formats: &PucchFormatsConfig) -> &[u32] {
        let num_sym = match &self.format {
            PucchFormat::Format3 { num_rb: _, num_sym, .. } => *num_sym,
            PucchFormat::Format4 { num_sym, .. } => *num_sym,
            _ => panic!("impossible to be here!"),
        };

        if num_sym == 4 {
            match self.intra_slot_freq_hopping {
                IntraSlotFreqHopping::Hopping { .. } => &[0, 2],
                IntraSlotFreqHopping::NoHopping => &[1],
            }
        } else {
            const START_SYM: u32 = 5;
            const NUM_TABLE: usize = (NUM_SYM_PER_SLOT - START_SYM + 1) as usize;
            let pucch_format_config = self.pucch_format_config(pucch_formats);
            let idx: usize = (num_sym - START_SYM) as usize;
            if pucch_format_config.addition_dmrs {
                #[rustfmt::skip]
                const PUCCH_DMRS_POS_TABLE: [&'static [u32]; NUM_TABLE] = [&[0, 3], &[1, 4], &[1, 4], &[1, 5], &[1, 6], &[1, 3, 6, 8], &[1, 3, 6, 9], &[1, 4, 7, 10], &[1, 4, 7, 11], &[1, 5, 8, 12]];
                PUCCH_DMRS_POS_TABLE[idx]
            } else {
                #[rustfmt::skip]
                const PUCCH_DMRS_POS_TABLE: [&'static [u32]; NUM_TABLE] = [&[0, 3], &[1, 4], &[1, 4], &[1, 5], &[1, 6], &[2, 7], &[2, 7], &[2, 8], &[2, 9], &[3, 10]];
                PUCCH_DMRS_POS_TABLE[idx]
            }
        }
    }

    // 38.213, 9.2.5.2
    fn max_hold_bits(&self, pucch_formats: &PucchFormatsConfig) -> u32 {
        match &self.format {
            PucchFormat::Format0 { .. } => 2,
            PucchFormat::Format1 { .. } => 2,
            _ => {
                let pucch_format_config = self.pucch_format_config(pucch_formats);
                let num_bits = match &self.format {
                    PucchFormat::Format2 { num_rb, num_sym, .. } => {
                        let data_sc = NUM_SC_PER_RB - 4;
                        let qm = QPSK_BITS;
                        pucch_format_config.max_coderate_x100 * (*num_rb) * data_sc * (*num_sym) * qm
                    }
                    PucchFormat::Format3 { num_rb, num_sym, .. } => {
                        let data_sc = NUM_SC_PER_RB;
                        let num_data_sym = *num_sym - self.pucch_num_dmrs_sym(pucch_formats);
                        let qm = if pucch_format_config.pi2_bpsk { BPSK_BITS } else { QPSK_BITS };
                        pucch_format_config.max_coderate_x100 * (*num_rb) * data_sc * num_data_sym * qm
                    }
                    PucchFormat::Format4 { num_sym, occ_len, .. } => {
                        let data_sc = NUM_SC_PER_RB / *occ_len;
                        let num_data_sym = *num_sym - self.pucch_num_dmrs_sym(pucch_formats);
                        let qm = if pucch_format_config.pi2_bpsk { BPSK_BITS } else { QPSK_BITS };
                        pucch_format_config.max_coderate_x100 * data_sc * num_data_sym * qm
                    }
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
        self.pucch_resource_set.iter().find(|&set| o_uci <= set.max_payload_minus_1).unwrap()
    }

    fn pucch_resource_for_uci(
        &self,
        o_uci: u32,
        pucch_resource_indicator: u32,
        dci_first_cce: u32,
        num_cce: u32,
    ) -> &PucchResource {
        let pucch_resource_set = self.pucch_resource_set_for_uci(o_uci);
        let idx_in_set = match pucch_resource_set.pucch_resource_set_id {
            0 => {
                // 38.213, 9.2.3
                let num_pucch_resource = pucch_resource_set.pucch_resource_id.len() as u32;
                if num_pucch_resource > 8 {
                    let pucch_resource_idx = floor(dci_first_cce * ceil(num_pucch_resource, 8), num_cce)
                        + pucch_resource_indicator * ceil(num_pucch_resource, 8);
                    let num_pucch_resource_mod_8 = num_pucch_resource % 8;
                    if pucch_resource_indicator < num_pucch_resource_mod_8 {
                        pucch_resource_idx
                    } else {
                        pucch_resource_idx + num_pucch_resource_mod_8
                    }
                } else {
                    pucch_resource_indicator
                }
            }
            1..=3 => pucch_resource_indicator,
            _ => panic!("impossible to be here!"),
        };

        let pucch_resource_idx = pucch_resource_set.pucch_resource_id[idx_in_set as usize].idx;
        &self.pucch_resource[pucch_resource_idx]
    }
}

impl PucchResource {
    // (start_sym, num_sym)
    fn occupied_sym(&self) -> (u32, u32) {
        match self.format {
            PucchFormat::Format0 { init_cyclic_shift: _, num_sym, start_sym } => (start_sym, num_sym),
            PucchFormat::Format1 { init_cyclic_shift: _, num_sym, start_sym, .. } => (start_sym, num_sym),
            PucchFormat::Format2 { num_rb: _, num_sym, start_sym } => (start_sym, num_sym),
            PucchFormat::Format3 { num_rb: _, num_sym, start_sym } => (start_sym, num_sym),
            PucchFormat::Format4 { num_sym, occ_len: _, occ_idx: _, start_sym } => (start_sym, num_sym),
        }
    }

    fn sym_bitmap(&self) -> u32 {
        let (start_sym, num_sym) = self.occupied_sym();
        mask(start_sym, num_sym)
    }

    fn is_overlap(&self, pucch_resource: &PucchResource) -> bool {
        (self.sym_bitmap() & pucch_resource.sym_bitmap()) != 0
    }

    fn pucch_format_type(format: &PucchFormat) -> PucchFormatType {
        match format {
            PucchFormat::Format0 { .. } | PucchFormat::Format2 { .. } => PucchFormatType::ShortPucch,
            _ => PucchFormatType::LongPucch,
        }
    }
}

#[derive(Debug)]
struct PucchLogicChannel {
    channel_type: PucchChannelType,
    pucch_resource_id: PucchResourceId,
}

#[derive(Debug, Clone, PartialEq)]
struct CsiReport {
    priority: u32,
    o_csi: u32, // csi payload size
    o_csi_1: u32,
    o_csi_2: Option<u32>,
}

#[derive(Debug, PartialEq)]
struct SrRequest {
    positive: bool,
    sr_id: u32,
}

#[derive(Debug, PartialEq)]
enum PucchChannelType {
    HarqDci,
    HarqSps,
    Sr(SrRequest),
    Csi(CsiReport),
    CsiMulti(Vec<CsiReport>),
    HarqSrMulti,
    HarqCsiMulti,
    CsiSrMulti,
    HarqCsiSrMulti,
}

impl PucchChannelType {
    check_func!(is_csi, PucchChannelType::Csi { .. });
    check_func!(is_sr, PucchChannelType::Sr { .. });
    check_func!(is_harq, PucchChannelType::HarqDci, PucchChannelType::HarqSps);
    check_func!(
        has_harq,
        PucchChannelType::HarqDci,
        PucchChannelType::HarqSps,
        PucchChannelType::HarqSrMulti,
        PucchChannelType::HarqCsiMulti,
        PucchChannelType::HarqCsiSrMulti
    );

    fn has_sr(&self) -> bool {
        unimplemented!()
    }

    fn has_csi(&self) -> bool {
        unimplemented!()
    }

    fn is_multi(&self) -> bool {
        unimplemented!()
    }
}

impl PucchLogicChannel {
    fn pucch_resource<'a>(&'a self, pucch_config: &'a PucchConfig) -> &'a PucchResource {
        self.pucch_resource_id.pucch_resource(pucch_config)
    }

    fn is_overlap<'a, T>(pucch_config: &'a PucchConfig, pucch_channels: T) -> bool
    where
        T: Iterator<Item = &'a PucchLogicChannel>,
    {
        pucch_channels
            .scan(0u32, |accum_bitmap, channel| {
                let pucch_bitmap = channel.pucch_resource(pucch_config).sym_bitmap();
                let is_overlap = (*accum_bitmap & pucch_bitmap) != 0;
                *accum_bitmap |= pucch_bitmap;
                Some(is_overlap)
            })
            .any(|is_overlap| is_overlap)
    }
}

fn pucch_proc(pucch_config: &PucchConfig, pucch_logic_channel: &mut Vec<PucchLogicChannel>) {
    // 1. if DCI harq exist, remove the sps harq

    // 2. deal with CSI pucch, either multiplex or select, at most 2 csi PUCCH will be remained
    csi_pucch_proc(pucch_config, pucch_logic_channel);

    // 3. if simultaneousHARQ-ACK-CSI not configured
    // 1) drop any CSI overlapped with HARQ-ACK
    // 2) if HARQ-ACK PUCCH is long PUCCH, drop all non-overlapped CSI with long PUCCH
    harq_csi_simul_proc(pucch_config, pucch_logic_channel);

    // 4. drop all negetive SR which is not overlap with any HARQ/CSI
    sr_pucch_proc(pucch_config, pucch_logic_channel);

    // 5. Q set process
}

fn csi_pucch_proc(pucch_config: &PucchConfig, pucch_logic_channel: &mut Vec<PucchLogicChannel>) {
    let csi_pucch_idx = filter_index(pucch_logic_channel, |channel| channel.channel_type.is_csi());

    if csi_pucch_idx.len() > 1 {
        match &pucch_config.multi_csi_resource {
            Some(mult_csi_resources) => {
                let csi_pucch = csi_pucch_idx.iter().map(|&i| &pucch_logic_channel[i]);
                let is_overlap = PucchLogicChannel::is_overlap(pucch_config, csi_pucch);
                // if multi_csi_resources is configured, and there's overlap between csi PUCCH,
                // then multiplex all CSI reports on one PUCCH from multi_csi_resources
                // otherwise, select one or two CSI pucch to transmit
                if is_overlap {
                    multi_csi_pucch_proc(pucch_config, pucch_logic_channel, csi_pucch_idx);
                } else {
                    select_csi_pucch_proc(pucch_config, pucch_logic_channel, csi_pucch_idx);
                }
            }
            None => {
                select_csi_pucch_proc(pucch_config, pucch_logic_channel, csi_pucch_idx);
            }
        }
    }
}

// multiplex all csi reports on one PUCCH from multi_csi_resources
fn multi_csi_pucch_proc(
    pucch_config: &PucchConfig,
    pucch_logic_channel: &mut Vec<PucchLogicChannel>,
    csi_pucch_idx: Vec<usize>,
) {
    let all_csi_reports = csi_pucch_idx
        .iter()
        .map(|&idx| into_variant!(pucch_logic_channel[idx].channel_type, PucchChannelType::Csi))
        .cloned()
        .collect::<Vec<_>>();

    let o_csi_all_reports = all_csi_reports.iter().fold(0u32, |o_csi_sum, csi_report| o_csi_sum + csi_report.o_csi);

    let resource_id = pucch_config.multi_csi_resource.as_ref().unwrap();
    let last_resource_id = resource_id.last().unwrap();
    // find the first (smallest) resource which can hold all the csi payload, if not found, take the last (largest) one
    // pucch_config.multi_csi_resource already been sorted, the capacity is from small to large
    let csi_pucch_id = resource_id
        .iter()
        .find(|&id| o_csi_all_reports <= id.pucch_resource(pucch_config).max_hold_bits)
        .unwrap_or(last_resource_id);

    // drop all CSI PUCCH, then add a CSI PUCCH to multiplex all CSI reports
    pucch_logic_channel.retain(|channel| !channel.channel_type.is_csi());
    pucch_logic_channel.push(PucchLogicChannel {
        channel_type: PucchChannelType::CsiMulti(all_csi_reports),
        pucch_resource_id: csi_pucch_id.clone(),
    });
}

// select one or two CSI pucch to transmit
fn select_csi_pucch_proc(
    pucch_config: &PucchConfig,
    pucch_logic_channel: &mut Vec<PucchLogicChannel>,
    csi_channel_idx: Vec<usize>,
) {
    // the lower channel priority value, the higher priority
    let highest_priority_csi_channel_idx = *csi_channel_idx
        .iter()
        .min_by_key(|&&idx| {
            let csi_report = into_variant!(pucch_logic_channel[idx].channel_type, PucchChannelType::Csi);
            csi_report.priority
        })
        .unwrap();
    let highest_pucch_resource = pucch_logic_channel[highest_priority_csi_channel_idx].pucch_resource(pucch_config);

    // the second CSI should be the second priority CSI which fulfill:
    // (1) do not overlap with highest priority CSI PUCCH
    // (2) if highest priority CSI is long PUCCH, it should be short PUCCH
    let second_csi_channel_idx = match highest_pucch_resource.format_type {
        PucchFormatType::LongPucch => csi_channel_idx
            .iter()
            .filter(|&&idx| {
                let resource = pucch_logic_channel[idx].pucch_resource(pucch_config);
                (idx != highest_priority_csi_channel_idx)
                    && resource.format_type == PucchFormatType::ShortPucch
                    && !resource.is_overlap(highest_pucch_resource)
            })
            .min_by_key(|&&idx| {
                let csi_report = into_variant!(pucch_logic_channel[idx].channel_type, PucchChannelType::Csi);
                csi_report.priority
            }),
        PucchFormatType::ShortPucch => csi_channel_idx
            .iter()
            .filter(|&&idx| {
                let resource = pucch_logic_channel[idx].pucch_resource(pucch_config);
                (idx != highest_priority_csi_channel_idx) && !resource.is_overlap(highest_pucch_resource)
            })
            .min_by_key(|&&idx| {
                let csi_report = into_variant!(pucch_logic_channel[idx].channel_type, PucchChannelType::Csi);
                csi_report.priority
            }),
    };

    // for CSI, only retain highest/second priority CSI pucch, drop all the others
    match second_csi_channel_idx {
        Some(&second_idx) => pucch_logic_channel.retain(with_index(|idx, channel: &PucchLogicChannel| {
            !channel.channel_type.is_csi() || idx == second_idx || idx == highest_priority_csi_channel_idx
        })),
        None => pucch_logic_channel.retain(with_index(|idx, channel: &PucchLogicChannel| {
            !channel.channel_type.is_csi() || idx == highest_priority_csi_channel_idx
        })),
    }
}

// drop all negetive SR which is not overlap with any HARQ/CSI
fn sr_pucch_proc(_pucch_config: &PucchConfig, pucch_logic_channel: &mut Vec<PucchLogicChannel>) {
    swap_remove_filter(pucch_logic_channel, |channel| {
        as_variant!(channel.channel_type, PucchChannelType::Sr).and_then(|sr| Some(!sr.positive)).unwrap_or(false)
    });
}

fn harq_csi_simul_proc(pucch_config: &PucchConfig, pucch_logic_channel: &mut Vec<PucchLogicChannel>) {}
