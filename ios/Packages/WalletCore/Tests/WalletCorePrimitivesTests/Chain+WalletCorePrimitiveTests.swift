// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing
import WalletCore
import WalletCorePrimitives

final class Chain_WalletCorePrimitiveTests {
    @Test(arguments: Chain.allCases)
    func chainToCoinType(chain: Chain) {
        let expected: CoinType = switch chain {
        case .bitcoin:
            .bitcoin
        case .litecoin:
            .litecoin
        case .ethereum, .smartChain, .polygon, .arbitrum, .optimism, .base,
             .avalancheC, .opBNB, .fantom, .gnosis, .manta, .blast, .zkSync,
             .linea, .mantle, .celo, .world, .sonic, .seiEvm, .abstract, .berachain,
             .ink, .unichain, .hyperliquid, .monad, .hyperCore, .plasma, .xLayer, .stable:
            .ethereum
        case .solana:
            .solana
        case .thorchain:
            .thorchain
        case .cosmos:
            .cosmos
        case .osmosis:
            .osmosis
        case .ton:
            .ton
        case .tron:
            .tron
        case .doge:
            .dogecoin
        case .aptos:
            .aptos
        case .sui:
            .sui
        case .xrp:
            .xrp
        case .celestia:
            .tia
        case .injective:
            .nativeInjective
        case .sei:
            .sei
        case .noble:
            .noble
        case .near:
            .near
        case .stellar:
            .stellar
        case .bitcoinCash:
            .bitcoinCash
        case .algorand:
            .algorand
        case .polkadot:
            .polkadot
        case .cardano:
            .cardano
        case .zcash:
            .zcash
        }

        #expect(chain.coinType == expected)
    }

    @Test
    func testIsValidAddress() {
        // Expect addresses to be valid
        #expect(Chain.mock(.ethereum).isValidAddress("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5"))
        #expect(Chain.mock(.ethereum).isValidAddress("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5"))

        // Expect addresses to be invalid
        #expect(!Chain.mock(.ethereum).isValidAddress("0x123"))
        #expect(!Chain.mock(.ethereum).isValidAddress("0x123"))
    }

    @Test
    func testChecksumAddress() {
        let bitocoinAddress = "bc1qr6f065nr70x4gl6ja9lm5wfj7xkhdv2sq04q23"
        let evmAddress = "0xd41fdb03ba84762dd66a0af1a6c8540ff1ba5dfb"
        let evmChecksumAddress = "0xD41FDb03Ba84762dD66a0af1a6C8540FF1ba5dfb"

        #expect(Chain.mock(.ethereum).checksumAddress(evmAddress) == evmChecksumAddress)
        #expect(Chain.mock(.smartChain).checksumAddress(evmAddress) == evmChecksumAddress)
        #expect(Chain.mock(.ethereum).checksumAddress(evmChecksumAddress) == evmChecksumAddress)
        #expect(Chain.mock(.bitcoin).checksumAddress(bitocoinAddress) == bitocoinAddress)
    }
}
