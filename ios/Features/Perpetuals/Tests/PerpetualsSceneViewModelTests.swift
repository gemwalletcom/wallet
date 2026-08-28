// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import protocol Gemstone.GemPerpetualServiceProtocol
import Store
import StoreTestKit
import GemstoneServices
import GemstoneServicesTestKit
@testable import Perpetuals
import PerpetualsTestKit
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct PerpetualsSceneViewModelTests {
    @Test
    func headerViewModel() {
        let wallet = Wallet.mock(type: .multicoin)
        let model = PerpetualsSceneViewModel.mock(wallet: wallet)

        #expect(model.headerViewModel.walletType == .multicoin)
    }
}

extension PerpetualsSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        perpetualService: any GemPerpetualServiceProtocol = GemPerpetualServiceMock(),
        observerService: any PerpetualObservable = PerpetualObserverMock(),
        recentActivityStore: RecentActivityStore = .mock(),
    ) -> PerpetualsSceneViewModel {
        PerpetualsSceneViewModel(
            wallet: wallet,
            perpetualService: perpetualService,
            observerService: observerService,
            recentActivityStore: recentActivityStore,
        )
    }
}
