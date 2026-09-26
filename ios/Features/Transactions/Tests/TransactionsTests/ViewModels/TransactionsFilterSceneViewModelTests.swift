// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Testing
@testable import Transactions
import TransactionsTestKit

@MainActor
struct TransactionsFilterSceneViewModelTests {
    @Test
    func aChainSelectionReachesTheRequest() {
        let model = TransactionsFilterSceneViewModel.mock()

        let confirmed = model.onFinishChainsSelection(SelectionResult(items: [.bitcoin], isConfirmed: true))

        #expect(confirmed)
        #expect(model.networksModel.selectedItems == [.bitcoin])
        #expect(model.query.request.base.filter?.chains == [.bitcoin])
        #expect(model.isAnyFilterSpecified)
    }

    @Test
    func aCancelledSelectionStillKeepsWhatWasPicked() {
        let model = TransactionsFilterSceneViewModel.mock()

        let confirmed = model.onFinishChainsSelection(SelectionResult(items: [.ethereum], isConfirmed: false))

        #expect(confirmed == false)
        #expect(model.networksModel.selectedItems == [.ethereum])
    }

    @Test
    func theTypesComeFromCore() {
        let model = TransactionsFilterSceneViewModel.mock()

        #expect(model.viewState.types.isNotEmpty)
        #expect(model.isAnyFilterSpecified == false)
    }

    @Test
    func aTypeSelectionMapsToItsTransactionTypes() {
        let model = TransactionsFilterSceneViewModel.mock()
        guard let filter = model.viewState.types.first else {
            Issue.record("core returned no transaction filters")
            return
        }

        _ = model.onFinishTypesSelection(SelectionResult(items: [filter], isConfirmed: true))

        #expect(model.typesModel.selectedItems == [filter])
        #expect(model.isAnyFilterSpecified)

        model.onClear()

        #expect(model.isAnyFilterSpecified == false)
    }

    @Test
    func eachFilterSheetOpensOnItsOwn() {
        let model = TransactionsFilterSceneViewModel.mock()

        model.onSelectChainsFilter()
        #expect(model.isPresentingChains)
        #expect(model.isPresentingTypes == false)

        model.onSelectTypesFilter()
        #expect(model.isPresentingTypes)
    }
}
