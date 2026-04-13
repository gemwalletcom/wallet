// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import ActivityServiceTestKit
@testable import Perpetuals
import PerpetualService
import PerpetualServiceTestKit
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
        perpetualService: PerpetualServiceable = PerpetualService.mock(),
        observerService: any PerpetualObservable = PerpetualObserverMock(),
        activityService: ActivityService = .mock(),
    ) -> PerpetualsSceneViewModel {
        PerpetualsSceneViewModel(
            wallet: wallet,
            perpetualService: perpetualService,
            observerService: observerService,
            activityService: activityService,
        )
    }
}
