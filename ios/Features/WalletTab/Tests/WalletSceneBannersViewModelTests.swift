// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing
@testable import WalletTab

@MainActor
struct WalletSceneBannersViewModelTests {
    @Test
    func priorityBannerSkipsOnboardingWhenWalletHasBalance() {
        let model = WalletSceneBannersViewModel(
            banners: [
                .mock(event: .onboarding, state: .alwaysActive),
                .mock(event: .enableNotifications, state: .active),
            ],
            totalFiatValue: 42,
        )

        #expect(model.allBanners.first?.event == .enableNotifications)
    }
}
