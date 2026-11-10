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
    func aTypeSelectionMapsToItsTransactionTypes() {
        let model = TransactionsFilterSceneViewModel.mock()
        let unfiltered = model.query.request.base.filter?.transactionTypes

        _ = model.onFinishTypesSelection(SelectionResult(items: [.transfers], isConfirmed: true))

        #expect(model.typesModel.selectedItems == [.transfers])
        #expect(model.query.request.base.filter?.transactionTypes == [.transfer, .transferNFT])
        #expect(model.isAnyFilterSpecified)

        model.onClear()

        #expect(model.query.request.base.filter?.transactionTypes == unfiltered)
        #expect(model.isAnyFilterSpecified == false)
    }

    @Test
    func eachFilterSheetOpensOnItsOwn() {
        let model = TransactionsFilterSceneViewModel.mock()

        model.onSelectChainsFilter()
        #expect(model.isPresentingSheet == .chains)

        model.onSelectTypesFilter()
        #expect(model.isPresentingSheet == .types)
    }
}
