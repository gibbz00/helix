use anyhow::Result;

// This binary is used in the Release CI as an optimization to cut down on
// compilation time. This is not meant to be run manually.

fn main() -> Result<()> {
    self::grammar::fetch_grammars()
}
