// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServicesTestKit
import Observation
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing
@testable import WalletTab
import WalletTabTestKit

@MainActor
struct WalletSceneViewModelTests {
    @Test
    func renameNotifiesWalletBar() async throws {
        let wallet = Wallet.mock(id: .multicoin(address: "0x1"), name: "First")
        let db = try DB.mockWithWallets([wallet])
        let store = WalletStore.mock(db: db)

        let model = WalletSceneViewModel.mock(wallet: wallet, db: db)

        #expect(model.walletBarModel.name == "First")

        await confirmation(expectedCount: 1...) { changed in
            withObservationTracking {
                _ = model.wallet
            } onChange: {
                changed()
            }

            try? store.renameWallet(wallet.id, name: "Renamed")
            for _ in 0 ..< 100 where model.wallet.name != "Renamed" {
                try? await Task.sleep(for: .milliseconds(10))
            }
        }

        #expect(model.walletBarModel.name == "Renamed")
    }

    @Test
    func onboardingBannerShowsOnlyWhileEveryBalanceIsZero() throws {
        let fundedDB = DB.mockAssets()
        let emptyDB = DB.mockAssets(assets: [.mock()])
        let banner = NewBanner(id: "onboarding", walletId: Wallet.mock().id.id, event: .onboarding, state: .active)
        try BannerStore.mock(db: fundedDB).addBanners([banner])
        try BannerStore.mock(db: emptyDB).addBanners([banner])

        #expect(WalletSceneViewModel.mock(db: fundedDB).homeState.visibleBanners.map(\.event) == [])
        #expect(WalletSceneViewModel.mock(db: emptyDB).homeState.visibleBanners.map(\.event) == [.onboarding])
    }
}
