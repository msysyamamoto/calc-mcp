use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};

mod calculator;
#[cfg(test)]
mod calculator_tests;
use calculator::CalculatorService;

#[tokio::main]
async fn main() -> Result<()> {
    let service = CalculatorService.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
