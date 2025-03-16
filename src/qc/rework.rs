use qc_traits::{QcField, QcRework};

use crate::{Header, SP3};

impl QcRework for SP3 {
    fn add_mut(&mut self, field: &QcField) {
        self.header.add_mut(field);
    }
    fn remove_mut(&mut self, field: &QcField) {
        self.header.remove_mut(field);
    }
}

impl QcRework for Header {
    fn add_mut(&mut self, field: &QcField) {
        match field {
            QcField::Agency(agency) => {
                self.agency = agency.to_string();
            },
            _ => {}, // Not applicable
        }
    }

    fn remove_mut(&mut self, field: &QcField) {
        match field {
            QcField::Agency(_) => self.agency = "".to_string(),
            _ => {}, // Not applicable
        }
    }
}
