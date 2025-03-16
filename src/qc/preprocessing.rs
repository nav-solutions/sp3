use std::collections::HashMap;

use crate::prelude::{Constellation, Duration, Epoch, Header, SP3, SV};

use qc_traits::{
    QcDecimationFilter, QcFilter, QcFilterType, QcMaskOperand, QcPreprocessing, QcSubset,
};

fn header_mask_mut(header: &mut Header, subset: &QcSubset, mask: QcMaskOperand) {
    match mask {
        QcMaskOperand::Equals => match subset {
            QcSubset::Satellites(svnn) => {
                header.satellites.retain(|sv| svnn.contains(&sv));
            },
            QcSubset::Constellations(constells) => {
                let broad_sbas_filter = constells.contains(&Constellation::SBAS);
                header.satellites.retain(|sv| {
                    if broad_sbas_filter {
                        sv.constellation.is_sbas() || constells.contains(&sv.constellation)
                    } else {
                        constells.contains(&sv.constellation)
                    }
                });
            },
            _ => {},
        },
        QcMaskOperand::GreaterEquals => match subset {
            QcSubset::Satellites(svnn) => {
                header.satellites.retain(|sv| {
                    let mut retained = true;
                    for item in svnn.iter() {
                        if sv.constellation == item.constellation {
                            retained &= sv.prn >= item.prn;
                        }
                    }
                    retained
                });
            },
            _ => {},
        },
        QcMaskOperand::GreaterThan => match subset {
            QcSubset::Satellites(svnn) => {
                header.satellites.retain(|sv| {
                    let mut retained = true;
                    for item in svnn.iter() {
                        if sv.constellation == item.constellation {
                            retained &= sv.prn > item.prn;
                        }
                    }
                    retained
                });
            },
            _ => {},
        },
        QcMaskOperand::LowerEquals => match subset {
            QcSubset::Satellites(svnn) => {
                header.satellites.retain(|sv| {
                    let mut retained = true;
                    for item in svnn.iter() {
                        if sv.constellation == item.constellation {
                            retained &= sv.prn <= item.prn;
                        }
                    }
                    retained
                });
            },
            _ => {},
        },
        QcMaskOperand::LowerThan => match subset {
            QcSubset::Satellites(svnn) => {
                header.satellites.retain(|sv| {
                    let mut retained = true;
                    for item in svnn.iter() {
                        if sv.constellation == item.constellation {
                            retained &= sv.prn < item.prn;
                        }
                    }
                    retained
                });
            },
            _ => {},
        },
        QcMaskOperand::NotEquals => match subset {
            QcSubset::Satellites(svnn) => {
                header.satellites.retain(|sv| !svnn.contains(&sv));
            },
            QcSubset::Constellations(constells) => {
                let broad_sbas_filter = constells.contains(&Constellation::SBAS);
                header.satellites.retain(|sv| {
                    if broad_sbas_filter {
                        !sv.constellation.is_sbas() && !constells.contains(&sv.constellation)
                    } else {
                        !constells.contains(&sv.constellation)
                    }
                });
            },
            _ => {},
        },
    }
}

fn header_decim_mut(header: &mut Header, _: &QcSubset, decim: QcDecimationFilter) {
    match decim {
        QcDecimationFilter::Duration(interval) => {
            header.epoch_interval = std::cmp::max(header.epoch_interval, interval);
        },
        QcDecimationFilter::Modulo(modulo) => {
            header.epoch_interval = header.epoch_interval * modulo as f64;
        },
    }
}

impl QcPreprocessing for Header {
    fn filter_mut(&mut self, f: &QcFilter) {
        match f.filter {
            QcFilterType::Mask(mask) => header_mask_mut(self, &f.subset, mask),
            QcFilterType::Decimation(decim) => header_decim_mut(self, &f.subset, decim),
        }
    }
}

fn mask_mut(sp3: &mut SP3, subset: &QcSubset, mask: QcMaskOperand) {
    match mask {
        QcMaskOperand::Equals => match subset {
            QcSubset::Satellites(svnn) => {
                sp3.data.retain(|k, _| svnn.contains(&k.sv));
            },
            QcSubset::Constellations(constells) => {
                sp3.data
                    .retain(|k, _| constells.contains(&k.sv.constellation));
            },
            _ => {},
        },
        QcMaskOperand::GreaterEquals => match subset {
            QcSubset::Satellites(svnn) => {
                sp3.data.retain(|k, _| {
                    let mut retained = true;
                    for item in svnn.iter() {
                        if k.sv.constellation == item.constellation {
                            retained &= k.sv.prn >= item.prn;
                        }
                    }
                    retained
                });
            },
            _ => {},
        },
        QcMaskOperand::GreaterThan => match subset {
            QcSubset::Satellites(svnn) => {
                sp3.data.retain(|k, _| {
                    let mut retained = true;
                    for item in svnn.iter() {
                        if k.sv.constellation == item.constellation {
                            retained &= k.sv.prn > item.prn;
                        }
                    }
                    retained
                });
            },
            _ => {},
        },
        QcMaskOperand::LowerEquals => match subset {
            QcSubset::Satellites(svnn) => {
                sp3.data.retain(|k, _| {
                    let mut retained = true;
                    for item in svnn.iter() {
                        if k.sv.constellation == item.constellation {
                            retained &= k.sv.prn <= item.prn;
                        }
                    }
                    retained
                });
            },
            _ => {},
        },
        QcMaskOperand::LowerThan => match subset {
            QcSubset::Satellites(svnn) => {
                sp3.data.retain(|k, _| {
                    let mut retained = true;
                    for item in svnn.iter() {
                        if k.sv.constellation == item.constellation {
                            retained &= k.sv.prn < item.prn;
                        }
                    }
                    retained
                });
            },
            _ => {},
        },
        QcMaskOperand::NotEquals => match subset {
            QcSubset::Satellites(svnn) => {
                sp3.data.retain(|k, _| !svnn.contains(&k.sv));
            },
            QcSubset::Constellations(constells) => {
                sp3.data
                    .retain(|k, _| !constells.contains(&k.sv.constellation));
            },
            _ => {},
        },
    }
}

fn dt_decim_mut(s: &mut SP3, subset: &QcSubset, dt: Duration) {
    let mut epochs = HashMap::<SV, Epoch>::new();

    match subset {
        QcSubset::All => {
            s.data.retain(|k, _| {
                if let Some(prev) = epochs.get_mut(&k.sv) {
                    let interval = k.epoch - *prev;
                    if interval >= dt {
                        *prev = k.epoch;
                        true
                    } else {
                        false
                    }
                } else {
                    epochs.insert(k.sv, k.epoch);
                    true
                }
            });
        },
        QcSubset::Satellites(svnn) => {
            s.data.retain(|k, _| {
                if svnn.contains(&k.sv) {
                    if let Some(prev) = epochs.get_mut(&k.sv) {
                        let interval = k.epoch - *prev;
                        if interval >= dt {
                            *prev = k.epoch;
                            true
                        } else {
                            false
                        }
                    } else {
                        epochs.insert(k.sv, k.epoch);
                        true
                    }
                } else {
                    true
                }
            });
        },
        QcSubset::Constellations(constells) => {
            s.data.retain(|k, _| {
                if constells.contains(&k.sv.constellation) {
                    if let Some(prev) = epochs.get_mut(&k.sv) {
                        let interval = k.epoch - *prev;
                        if interval >= dt {
                            *prev = k.epoch;
                            true
                        } else {
                            false
                        }
                    } else {
                        epochs.insert(k.sv, k.epoch);
                        true
                    }
                } else {
                    true
                }
            });
        },
        _ => {},
    }
}

fn modulo_decim_mut(sp3: &mut SP3, subset: &QcSubset, modulo: u32) {
    let mut counters = HashMap::<SV, u32>::new();
    match subset {
        QcSubset::All => {
            sp3.data.retain(|k, _| {
                if let Some(count) = counters.get_mut(&k.sv) {
                    let retained = (*count % modulo) == 0;
                    *count += 1;
                    retained
                } else {
                    counters.insert(k.sv, 0);
                    true
                }
            });
        },
        QcSubset::Satellites(svnn) => {
            sp3.data.retain(|k, _| {
                if svnn.contains(&k.sv) {
                    if let Some(count) = counters.get_mut(&k.sv) {
                        let retained = (*count % modulo) == 0;
                        *count += 1;
                        retained
                    } else {
                        counters.insert(k.sv, 0);
                        true
                    }
                } else {
                    true
                }
            });
        },
        QcSubset::Constellations(constells) => {
            sp3.data.retain(|k, _| {
                if constells.contains(&k.sv.constellation) {
                    if let Some(count) = counters.get_mut(&k.sv) {
                        let retained = (*count % modulo) == 0;
                        *count += 1;
                        retained
                    } else {
                        counters.insert(k.sv, 0);
                        true
                    }
                } else {
                    true
                }
            });
        },
        _ => {},
    }
}

fn decim_mut(sp3: &mut SP3, subset: &QcSubset, decim: QcDecimationFilter) {
    match decim {
        QcDecimationFilter::Duration(dt) => dt_decim_mut(sp3, subset, dt),
        QcDecimationFilter::Modulo(modulo) => modulo_decim_mut(sp3, subset, modulo),
    }
}

impl QcPreprocessing for SP3 {
    fn filter_mut(&mut self, f: &QcFilter) {
        self.header.filter_mut(f);
        match f.filter {
            QcFilterType::Mask(mask) => mask_mut(self, &f.subset, mask),
            QcFilterType::Decimation(decim) => decim_mut(self, &f.subset, decim),
        }
    }
}
