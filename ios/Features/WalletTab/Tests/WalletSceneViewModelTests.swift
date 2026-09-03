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
        let db = DB.mock()
        let store = WalletStore.mock(db: db)
        try store.addWallet(wallet)

        let model = WalletSceneViewModel.mock(wallet: wallet)
        model.walletQuery.bind(dbQueue: db.dbQueue)

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
        let funded = try onboardingModel(db: DB.mockAssets())
        let empty = try onboardingModel(db: DB.mockAssets(assets: [.mock()]))

        #expect(funded.visibleBanners.map(\.event) == [])
        #expect(empty.visibleBanners.map(\.event) == [.onboarding])
    }

    private func onboardingModel(db: DB) throws -> WalletSceneViewModel {
        let wallet = Wallet.mock()
        try BannerStore(db: db).addBanners([NewBanner(id: "onboarding", walletId: wallet.id.id, event: .onboarding, state: .active)])
        let model = WalletSceneViewModel.mock(wallet: wallet)
        model.assetsQuery.bind(dbQueue: db.dbQueue)
        model.bannersQuery.bind(dbQueue: db.dbQueue)
        return model
    }
}
