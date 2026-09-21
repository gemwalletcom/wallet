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
    @Test(.timeLimit(.minutes(1)))
    func renameNotifiesWalletBar() async throws {
        let wallet = Wallet.mock(id: .multicoin(address: "0x1"), name: "First")
        let db = try DB.mockWithWallets([wallet])
        let store = WalletStore.mock(db: db)

        let model = WalletSceneViewModel.mock(wallet: wallet, db: db)

        #expect(model.walletBarModel.name == "First")

        try store.renameWallet(wallet.id, name: "Renamed")
        while model.wallet.name != "Renamed" {
            await withCheckedContinuation { changed in
                withObservationTracking { _ = model.wallet } onChange: { changed.resume() }
            }
        }

        #expect(model.walletBarModel.name == "Renamed")
    }
}
