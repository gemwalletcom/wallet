// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
import GemstoneServicesTestKit
import Onboarding
import Primitives
import PrimitivesTestKit

public extension SetupWalletViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        onComplete: @escaping (Wallet) -> Void = { _ in },
    ) -> SetupWalletViewModel {
        SetupWalletViewModel(
            wallet: wallet,
            service: GemWalletService.mock(),
            onSelectImage: { _ in },
            onComplete: onComplete,
        )
    }
}
