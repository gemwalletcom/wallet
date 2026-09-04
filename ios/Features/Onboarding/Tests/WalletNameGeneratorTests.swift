// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Onboarding
import Primitives
import class Gemstone.GemWalletService
import Testing
import GemstoneServices
import GemstoneServicesTestKit

struct WalletNameGeneratorTests {
    @Test
    func walletNameUsesNextWalletIndex() async {
        let generator = WalletNameGenerator(type: .multicoin, service: GemWalletService.mock())

        #expect(await generator.name() == "Wallet #1")
    }
}
