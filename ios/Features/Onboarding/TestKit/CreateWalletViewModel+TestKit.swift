// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
import protocol Gemstone.GemWalletServiceProtocol
import GemstoneServicesTestKit
import Onboarding

public extension CreateWalletViewModel {
    static func mock(service: any GemWalletServiceProtocol = GemWalletService.mock()) -> CreateWalletViewModel {
        CreateWalletViewModel(
            service: service,
            preferences: .mock(),
            onComplete: nil,
        )
    }
}
