// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
import protocol Gemstone.GemWalletServiceProtocol
import GemstoneServices
import GemstoneServicesTestKit
import Onboarding

public extension CreateWalletViewModel {
    static func mock(
        service: any GemWalletServiceProtocol = GemWalletService.mock(),
        preferences: ObservablePreferences = .mock(),
    ) -> CreateWalletViewModel {
        CreateWalletViewModel(
            service: service,
            biometryService: BiometryAuthenticationMock(),
            preferences: preferences,
            onComplete: nil,
        )
    }
}
