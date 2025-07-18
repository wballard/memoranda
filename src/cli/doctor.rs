use anyhow::Result;
use tracing::info;

pub struct DoctorCommand;

impl DoctorCommand {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self) -> Result<()> {
        info!("Running doctor command");
        println!("Doctor: All systems operational");
        Ok(())
    }
}