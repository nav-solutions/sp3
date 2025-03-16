use qc_traits::{QcFilter, QcFilterType, QcMaskOperand, QcPreprocessing, QcSubset};

use crate::{Header, SP3};

impl Masking for Header {
    fn mask_mut(&mut self, mask: &MaskFilter) {
        match mask.operand {
            MaskOperand::Equals => match &mask.item {
                FilterItem::EpochItem(epoch) => {
                    let mut mjd = Epoch::from_mjd_utc(self.mjd);
                    if self.timescale.is_gnss() {
                        mjd = Epoch::from_duration(
                            mjd - self.timescale.reference_epoch(),
                            self.timescale,
                        );
                    }

                    if *epoch < mjd {
                        // TODO
                    }
                },
                FilterItem::SvItem(svs) => {
                    self.satellites.retain(|sv| svs.contains(&sv));
                },
                FilterItem::ConstellationItem(constellations) => {
                    self.satellites
                        .retain(|sv| constellations.contains(&sv.constellation));
                },
                FilterItem::DurationItem(dt) => {
                    self.epoch_interval = std::cmp::max(self.epoch_interval, *dt);
                },
                _ => {},
            },
            MaskOperand::GreaterEquals => match &mask.item {
                FilterItem::SvItem(svs) => {
                    self.satellites.retain(|sv| {
                        let mut retained = true;
                        for item in svs {
                            if item.constellation == sv.constellation {
                                retained &= sv.prn >= item.prn;
                            }
                        }
                        retained
                    });
                },
                _ => {},
            },
            MaskOperand::GreaterThan => match &mask.item {
                FilterItem::SvItem(svs) => {
                    self.satellites.retain(|sv| {
                        let mut retained = true;
                        for item in svs {
                            if item.constellation == sv.constellation {
                                retained &= sv.prn > item.prn;
                            }
                        }
                        retained
                    });
                },
                _ => {},
            },
            MaskOperand::LowerEquals => match &mask.item {
                FilterItem::SvItem(svs) => {
                    self.satellites.retain(|sv| {
                        let mut retained = true;
                        for item in svs {
                            if item.constellation == sv.constellation {
                                retained &= sv.prn <= item.prn;
                            }
                        }
                        retained
                    });
                },
                _ => {},
            },
            MaskOperand::LowerThan => match &mask.item {
                FilterItem::SvItem(svs) => {
                    self.satellites.retain(|sv| {
                        let mut retained = true;
                        for item in svs {
                            if item.constellation == sv.constellation {
                                retained &= sv.prn < item.prn;
                            }
                        }
                        retained
                    });
                },
                _ => {},
            },
            MaskOperand::NotEquals => match &mask.item {
                FilterItem::SvItem(svs) => {
                    self.satellites.retain(|sv| {
                        let mut retained = true;
                        for item in svs {
                            if item.constellation == sv.constellation {
                                retained &= sv.prn != item.prn;
                            }
                        }
                        retained
                    });
                },
                _ => {},
            },
        }
    }
    fn mask(&self, mask: &MaskFilter) -> Self {
        let mut s = self.clone();
        s.mask_mut(mask);
        s
    }
}

impl Masking for SP3 {
    fn mask(&self, f: &MaskFilter) -> Self {
        let mut s = self.clone();
        s.mask_mut(&f);
        s
    }
    fn mask_mut(&mut self, f: &MaskFilter) {
        match f.operand {
            MaskOperand::Equals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.data.retain(|k, _| k.epoch == *epoch);
                },
                FilterItem::SvItem(svs) => {
                    self.data.retain(|k, _| svs.contains(&k.sv));
                },
                FilterItem::ConstellationItem(constells) => {
                    let mut broad_sbas_filter = false;
                    for c in constells {
                        broad_sbas_filter |= *c == Constellation::SBAS;
                    }
                    self.data.retain(|k, _| {
                        if broad_sbas_filter {
                            k.sv.constellation.is_sbas() || constells.contains(&k.sv.constellation)
                        } else {
                            constells.contains(&k.sv.constellation)
                        }
                    });
                },
                _ => {}, // does not apply
            },
            MaskOperand::NotEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.data.retain(|k, _| k.epoch != *epoch);
                },
                FilterItem::SvItem(svs) => {
                    self.data.retain(|k, _| !svs.contains(&k.sv));
                },
                FilterItem::ConstellationItem(constells) => {
                    self.data
                        .retain(|k, _| !constells.contains(&k.sv.constellation));
                },
                _ => {}, // does not apply
            },
            MaskOperand::GreaterThan => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.data.retain(|k, _| k.epoch > *epoch);
                },
                FilterItem::SvItem(svs) => {
                    self.data.retain(|k, _| {
                        let mut retain = false;
                        for sv in svs {
                            if k.sv.constellation == sv.constellation {
                                retain = k.sv.prn > sv.prn
                            } else {
                                retain = false
                            }
                        }
                        retain
                    });
                },
                _ => {}, // does not apply
            },
            MaskOperand::GreaterEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.data.retain(|k, _| k.epoch >= *epoch);
                },
                FilterItem::SvItem(svs) => {
                    self.data.retain(|k, _| {
                        let mut retain = false;
                        for sv in svs {
                            if k.sv.constellation == sv.constellation {
                                retain = k.sv.prn >= sv.prn
                            } else {
                                retain = false
                            }
                        }
                        retain
                    });
                },
                _ => {}, // does not apply
            },
            MaskOperand::LowerThan => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.data.retain(|k, _| k.epoch < *epoch);
                },
                FilterItem::SvItem(svs) => {
                    self.data.retain(|k, _| {
                        let mut retain = false;
                        for sv in svs {
                            if k.sv.constellation == sv.constellation {
                                retain = k.sv.prn < sv.prn
                            } else {
                                retain = false
                            }
                        }
                        retain
                    });
                },
                _ => {}, // does not apply
            },
            MaskOperand::LowerEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.data.retain(|k, _| k.epoch <= *epoch);
                },
                FilterItem::SvItem(svs) => {
                    self.data.retain(|k, _| {
                        let mut retain = false;
                        for sv in svs {
                            if k.sv.constellation == sv.constellation {
                                retain = k.sv.prn <= sv.prn
                            } else {
                                retain = false
                            }
                        }
                        retain
                    });
                },
                _ => {}, // does not apply
            },
        }
    }
}

fn header_mask_mut(header: &mut Header, subset: QcSubset, mask: QcMaskOperand) {

}

fn header_decim_mut(header: &mut Header, subset: QcSubset, decim: QcDecimationFilter) {
    match decim {
        QcDecimationFilterType::Duration(interval) => {
            self.epoch_interval = std::cmp::max(self.epoch_interval, interval);
        },
        QcDecimationFilterType::Modulo(modulo) => {
            self.epoch_interval = self.epoch_interval * modulo as f64;
        },
    }
}

impl QcPreprocessing for Header {
    fn filter_mut(&mut self, f: QcFilter) {
        match f.filter {
            QcFilterType::Mask(mask) => header_mask_mut(self, f.subset, mask),
            QcFilterType::Decimation(decim) => header_decim_mut(self, f.subset, mask),
        }
    }
}

fn mask_mut(s: &mut SP3, subset: QcSubset, mask: QcMaskOperand) {
}

fn decim_mut(s: &mut SP3, subset: QcSubset, mask: QcMaskOperand) {
}

impl QcPreprocessing for SP3 {
    fn filter_mut(&mut self, f: QcFilter) {
        self.header.filter_mut(f);
        match f.filter {
            QcFilterType::Mask(mask) => mask_mut(self, f.subset, mask),
            QcFilterType::Decimation(decim) => decim_mut(self, f.subset, mask),
        }
    }
}
