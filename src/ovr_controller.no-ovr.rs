use std::fmt::{Display, Formatter};
use std::path::Path;

pub struct OVRController {}

impl OVRController {
    pub fn new(_resources: &Path) -> Result<OVRController, OVRError> {
        Ok(Self {})
    }
}

#[derive(Debug)]
pub enum OVRError {}

impl Display for OVRError {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}
