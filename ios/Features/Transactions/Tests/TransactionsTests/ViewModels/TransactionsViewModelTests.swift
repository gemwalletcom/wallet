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
struct TransactionsViewModelTests {
    @Test
    func theFilterOffersTheChainsCoreReturned() {
        let service = GemTransactionsServiceMock(filterChains: [Chain.bitcoin.rawValue, Chain.ethereum.rawValue])
        let model = TransactionsViewModel.mock(service: service)

        #expect(model.filterModel.chainsFilter.allChains == [.bitcoin, .ethereum])
        #expect(model.filterModel.isAnyFilterSpecified == false)
    }

    @Test
    func loadingSyncsEveryAsset() async {
        let service = GemTransactionsServiceMock()
        let model = TransactionsViewModel.mock(service: service)

        await model.load()

        #expect(service.syncedAssetIds.count == 1)
        #expect(service.syncedAssetIds.first == .some(nil))
    }

    @Test
    func aFailedRefreshWithNothingShownShowsTheErrorInsteadOfTheEmptyState() async {
        let service = GemTransactionsServiceMock()
        let model = TransactionsViewModel.mock(service: service)

        await model.load()
        #expect(model.loadError == nil)

        service.refreshState = .error(error: .Gateway(msg: "offline"))
        await model.load()

        #expect(model.loadError != nil)
        #expect(model.isPresentingToastMessage == nil)
    }

    @Test
    func openingTheFilterPresentsTheSheet() {
        let model = TransactionsViewModel.mock()

        model.onSelectFilterButton()

        #expect(model.isPresentingSheet?.id == TransactionsSheetType.filter.id)
    }

    @Test
    func theEmptyStateChangesOnceAFilterIsOn() {
        let service = GemTransactionsServiceMock(filterChains: [Chain.bitcoin.rawValue])
        let model = TransactionsViewModel.mock(service: service)
        let unfiltered = model.emptyContentModel.title

        model.filterModel.chainsFilter.selectedChains = [.bitcoin]

        #expect(model.emptyContentModel.title != unfiltered)
    }
}

@MainActor
struct TransactionsFilterViewModelTests {
    @Test
    func aChainSelectionReachesTheRequest() {
        let model = TransactionsFilterViewModel.mock()

        let confirmed = model.onFinishChainsSelection(SelectionResult(items: [.bitcoin], isConfirmed: true))

        #expect(confirmed)
        #expect(model.chainsFilter.selectedChains == [.bitcoin])
        #expect(model.isAnyFilterSpecified)
    }

    @Test
    func aCancelledSelectionStillKeepsWhatWasPicked() {
        let model = TransactionsFilterViewModel.mock()

        let confirmed = model.onFinishChainsSelection(SelectionResult(items: [.ethereum], isConfirmed: false))

        #expect(confirmed == false)
        #expect(model.chainsFilter.selectedChains == [.ethereum])
    }

    @Test
    func theTypesComeFromCore() {
        let model = TransactionsFilterViewModel.mock()

        #expect(model.transactionTypesFilter.allTransactionsTypes.isNotEmpty)
        #expect(model.transactionTypesFilter.isAnySelected == false)
    }

    @Test
    func aTypeSelectionMapsToItsTransactionTypes() {
        let model = TransactionsFilterViewModel.mock()
        guard let filter = model.transactionTypesFilter.allTransactionsTypes.first else {
            Issue.record("core returned no transaction filters")
            return
        }

        _ = model.onFinishTypesSelection(SelectionResult(items: [filter], isConfirmed: true))

        #expect(model.transactionTypesFilter.selectedTypes == [filter])
        #expect(model.isAnyFilterSpecified)
    }

    @Test
    func eachFilterSheetOpensOnItsOwn() {
        let model = TransactionsFilterViewModel.mock()

        model.onSelectChainsFilter()
        #expect(model.isPresentingChains)
        #expect(model.isPresentingTypes == false)

        model.onSelectTypesFilter()
        #expect(model.isPresentingTypes)
    }
}
