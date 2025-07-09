use cedra_whitelist::initialize;

fn main() -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(initialize::initialize())
}
