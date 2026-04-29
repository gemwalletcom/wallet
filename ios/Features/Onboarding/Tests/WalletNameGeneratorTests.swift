// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Onboarding
import Primitives
import Testing
import WalletService
import WalletServiceTestKit

struct WalletNameGeneratorTests {
    @Test
    func walletNameWhenEmpty() {
        let generator = WalletNameGenerator(type: .multicoin, walletService: .mock())
        let name = ""

        let result = name.isEmpty ? generator.name : name

        #expect(result == "Wallet #1")
    }

    @Test
    func walletNameWhenProvided() {
        let generator = WalletNameGenerator(type: .multicoin, walletService: .mock())
        let name = "My Wallet"

        let result = name.isEmpty ? generator.name : name

        #expect(result == "My Wallet")
    }
}
