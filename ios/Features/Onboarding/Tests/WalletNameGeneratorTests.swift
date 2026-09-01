// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Onboarding
import Primitives
import class Gemstone.GemOnboardingService
import Testing
import GemstoneServices
import GemstoneServicesTestKit

struct WalletNameGeneratorTests {
    @Test
    func walletNameUsesNextWalletIndex() async {
        let generator = WalletNameGenerator(type: .multicoin, service: GemOnboardingService.mock())

        #expect(await generator.name() == "Wallet #1")
    }
}
