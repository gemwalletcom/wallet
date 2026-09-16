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

@MainActor
struct TransactionsViewModelTests {
    private func model(
        service: GemTransactionsServiceMock = GemTransactionsServiceMock(),
        wallet: Primitives.Wallet = .mock(),
    ) -> TransactionsViewModel {
        TransactionsViewModel(service: service, wallet: wallet, type: .all)
    }

    @Test
    func theFilterOffersTheChainsCoreReturned() {
        let service = GemTransactionsServiceMock(filterChains: [Chain.bitcoin.rawValue, Chain.ethereum.rawValue])
        let model = model(service: service)

        #expect(model.filterModel.chainsFilter.allChains == [.bitcoin, .ethereum])
        #expect(model.filterModel.isAnyFilterSpecified == false)
    }

    @Test
    func loadingSyncsEveryAsset() async {
        let service = GemTransactionsServiceMock()
        let model = model(service: service)

        await model.load()

        #expect(service.syncedAssetIds.count == 1)
        #expect(service.syncedAssetIds.first == .some(nil))
    }

    @Test
    func aFailedSyncLeavesNoToast() async {
        let service = GemTransactionsServiceMock()
        service.syncError = AnyError("offline")
        let model = model(service: service)

        await model.load()

        #expect(model.isPresentingToastMessage == nil)
    }

    @Test
    func openingTheFilterPresentsTheSheet() {
        let model = model()

        model.onSelectFilterButton()

        #expect(model.isPresentingSheet?.id == TransactionsSheetType.filter.id)
    }

    @Test
    func theEmptyStateChangesOnceAFilterIsOn() {
        let service = GemTransactionsServiceMock(filterChains: [Chain.bitcoin.rawValue])
        let model = model(service: service)
        let unfiltered = model.emptyContentModel.title

        model.filterModel.chainsFilter.selectedChains = [.bitcoin]

        #expect(model.emptyContentModel.title != unfiltered)
    }
}

@MainActor
struct TransactionsFilterViewModelTests {
    private func model(chains: [Primitives.Chain] = [.bitcoin, .ethereum]) -> TransactionsFilterViewModel {
        TransactionsFilterViewModel(wallet: .mock(), chains: chains, type: .all)
    }

    @Test
    func aChainSelectionReachesTheRequest() {
        let model = model()

        let confirmed = model.onFinishChainsSelection(SelectionResult(items: [.bitcoin], isConfirmed: true))

        #expect(confirmed)
        #expect(model.chainsFilter.selectedChains == [.bitcoin])
        #expect(model.isAnyFilterSpecified)
    }

    @Test
    func aCancelledSelectionStillKeepsWhatWasPicked() {
        let model = model()

        let confirmed = model.onFinishChainsSelection(SelectionResult(items: [.ethereum], isConfirmed: false))

        #expect(confirmed == false)
        #expect(model.chainsFilter.selectedChains == [.ethereum])
    }

    @Test
    func theTypesComeFromCore() {
        let model = model()

        #expect(model.transactionTypesFilter.allTransactionsTypes.isNotEmpty)
        #expect(model.transactionTypesFilter.isAnySelected == false)
    }

    @Test
    func aTypeSelectionMapsToItsTransactionTypes() {
        let model = model()
        guard let filter = model.transactionTypesFilter.allTransactionsTypes.first else {
            Issue.record("core returned no transaction filters")
            return
        }

        _ = model.onFinishTypesSelection(SelectionResult(items: [filter], isConfirmed: true))

        #expect(model.transactionTypesFilter.selectedTypes == [filter])
        #expect(model.transactionTypesFilter.requestFilters.isNotEmpty)
        #expect(model.isAnyFilterSpecified)
    }

    @Test
    func eachFilterSheetOpensOnItsOwn() {
        let model = model()

        model.onSelectChainsFilter()
        #expect(model.isPresentingChains)
        #expect(model.isPresentingTypes == false)

        model.onSelectTypesFilter()
        #expect(model.isPresentingTypes)
    }
}
