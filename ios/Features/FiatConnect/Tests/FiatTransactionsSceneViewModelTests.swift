// Copyright (c). Gem Wallet. All rights reserved.

@testable import FiatConnect
import func Gemstone.emptyState
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct FiatTransactionsSceneViewModelTests {
    @Test
    func aFailedRefreshWithNothingStoredShowsTheErrorInsteadOfTheEmptyState() async {
        let service = GemFiatQuoteServiceMock()
        let model = FiatTransactionsSceneViewModel(walletId: .mock(), service: service)

        #expect(model.phase == .empty(state: emptyState(kind: .activity)))

        service.refreshTransactionsState = .error(error: .Gateway(msg: "offline"))
        await model.load()

        #expect(model.phase == .error(error: .Gateway(msg: "offline")))
    }
}
