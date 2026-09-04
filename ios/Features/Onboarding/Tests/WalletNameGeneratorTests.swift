// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemWalletDefaultName
@testable import Onboarding
import Primitives
import Testing

struct WalletNameGeneratorTests {
    @Test
    func namesTheFirstMulticoinWallet() {
        let generator = WalletNameGenerator(defaultName: .multicoin(index: 1))

        #expect(generator.name() == "Wallet #1")
        #expect(generator.hasExistingWallets == false)
    }

    @Test
    func namesAChainWalletAndKnowsOthersExist() {
        let generator = WalletNameGenerator(defaultName: .chain(chain: Chain.bitcoin.rawValue, index: 3))

        #expect(generator.name() == "Bitcoin Wallet #3")
        #expect(generator.hasExistingWallets)
    }
}
