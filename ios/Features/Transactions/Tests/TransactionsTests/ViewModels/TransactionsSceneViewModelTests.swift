// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Gemstone
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing
@testable import Transactions
import TransactionsTestKit

@MainActor
struct TransactionsSceneViewModelTests {
    @Test
    func theFilterOffersTheChainsCoreReturned() {
        let service = GemTransactionsServiceMock(filterChains: [Chain.bitcoin.rawValue, Chain.ethereum.rawValue])
        let model = TransactionsSceneViewModel.mock(service: service)

        #expect(model.filterModel.networksModel.items == [.bitcoin, .ethereum])
        #expect(model.filterModel.isAnyFilterSpecified == false)
    }

    @Test
    func loadingSyncsEveryAsset() async {
        let service = GemTransactionsServiceMock()
        let model = TransactionsSceneViewModel.mock(service: service)

        await model.load()

        #expect(service.syncedAssetIds.count == 1)
        #expect(service.syncedAssetIds.first == .some(nil))
    }

    @Test
    func aFailedRefreshWithNothingShownShowsTheErrorInsteadOfTheEmptyState() async {
        let service = GemTransactionsServiceMock()
        let model = TransactionsSceneViewModel.mock(service: service)

        await model.load()
        #expect(model.loadError == nil)

        service.refreshState = .error(error: .Gateway(msg: "offline"))
        await model.load()

        #expect(model.loadError != nil)
        #expect(model.isPresentingToastMessage == nil)
    }

    @Test
    func openingTheFilterPresentsTheSheet() {
        let model = TransactionsSceneViewModel.mock()

        model.onSelectFilterButton()

        #expect(model.isPresentingSheet?.id == TransactionsSheetType.filter.id)
    }

    @Test
    func theEmptyStateChangesOnceAFilterIsOn() {
        let service = GemTransactionsServiceMock(filterChains: [Chain.bitcoin.rawValue])
        let model = TransactionsSceneViewModel.mock(service: service)
        let unfiltered = model.emptyContentModel.title

        _ = model.filterModel.onFinishChainsSelection(SelectionResult(items: [.bitcoin], isConfirmed: true))

        #expect(model.emptyContentModel.title != unfiltered)
    }
}
