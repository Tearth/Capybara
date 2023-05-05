use orion_core::anyhow::Result;
use orion_core::app::ApplicationContext;

fn main() -> Result<()> {
    ApplicationContext::new()?.run();
    Ok(())
}
