use bitcoin::ScriptBuf;
use gem_bitcoin::signer::SpendDestination;
use orchard::{
    Address as OrchardAddress,
    keys::{FullViewingKey, Scope, SpendingKey},
};
use primitives::BitcoinChain;

pub(crate) const TEST_ZCASH_UNIFIED_ADDRESS: &str = "u1qhwq6evpqq04ga6dn06mexlk6pcarhru4mhpkv63ar7nsmqvk5057lgtuj2yx6nhxlu0wuhzuw74v4q0eskt7gxy2xplq4qe5m560gukxtnx86mu5qljhgygyxzy7ulavg02qc42hxs";
pub(crate) const TEST_ZCASH_UNIFIED_ORCHARD_ADDRESS: &str = "u1u4zdvtcrjt4cnffw3shrx440754g0mhr4a7fenck686tfr6rdqy9wu5v39ydpgm4ut37qnlh9kpw9fsp8wcwyu2y2r5stjhj0qwfnmrq";
pub(crate) const TEST_ZCASH_UNIFIED_SAPLING_TRANSPARENT_ADDRESS: &str = "u169hujckwhkhn97vjhd546hc3v3wlt2qqc4f5f4fa0pqyg3ywfvrnw09ffk9vzql028eypyd5u9e2zn85l3jjekalxthuq3qkggukgtlwssfp03ffzvm50whggx7096k436hvg3gfpxx";
pub(crate) const TEST_ZCASH_UNIFIED_SAPLING_ADDRESS: &str = "u13vdyzej22x6cce5lgcdltm7m2jvmef3ny54phpdm5rgn77py2afnzfggshqwef2297r9rg4m0a6nel3qff0l6hfnmwvuw8nr3gls65s7";
pub(crate) const TEST_ZCASH_UNIFIED_TESTNET_ADDRESS: &str = "utest1twcrjuss3gvvhx926arfydv6n760x2akqxhfe0qg5kx3nprfqtp4dhgdw066jvm5n3l0xmu4w47xguyz7fnf4latqkqaqq7dughszayee6jz2j9asmwu494urxrzc7mfp7fnkl9w5dk";
const TEST_ORCHARD_SPENDING_KEY: [u8; 32] = [7u8; 32];

pub(crate) fn mock_orchard_recipient() -> OrchardAddress {
    FullViewingKey::from(&SpendingKey::from_bytes(TEST_ORCHARD_SPENDING_KEY).unwrap()).address_at(0u32, Scope::External)
}

pub(crate) fn mock_transparent_script(address: &str) -> ScriptBuf {
    let SpendDestination::Script(script) = SpendDestination::script(BitcoinChain::Zcash, address).unwrap() else {
        panic!("{address} is not a transparent address");
    };
    script
}
