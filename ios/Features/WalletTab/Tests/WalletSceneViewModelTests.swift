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
}
