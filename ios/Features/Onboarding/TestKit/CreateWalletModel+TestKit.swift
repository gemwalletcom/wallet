// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
import protocol Gemstone.GemWalletServiceProtocol
import GemstoneServicesTestKit
import Onboarding

public extension CreateWalletModel {
    static func mock(service: any GemWalletServiceProtocol = GemWalletService.mock()) -> CreateWalletModel {
        CreateWalletModel(
            service: service,
            preferences: .mock(),
            onComplete: nil,
        )
    }
}
