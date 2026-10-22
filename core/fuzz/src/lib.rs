use primitives::Chain;

pub fn chain(index: u8) -> Chain {
    let chains = Chain::all();
    chains[usize::from(index) % chains.len()]
}
