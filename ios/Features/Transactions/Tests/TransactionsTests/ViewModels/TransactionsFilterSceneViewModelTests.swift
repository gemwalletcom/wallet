// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemListWindow
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
@testable import Store
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
    func aFilterChangeStartsFromTheFirstPage() {
        let model = TransactionsFilterSceneViewModel.mock()
        let firstPage = { model.query.request.base.with(limit: GemListWindow.transactions.limit) }
        model.query.value = transactionListSections(Array(repeating: .mock(), count: GemListWindow.transactions.limit))
        model.onEndReached()
        #expect(model.query.request.base != firstPage())

        _ = model.onFinishChainsSelection(SelectionResult(items: [.bitcoin], isConfirmed: true))

        #expect(model.query.request.base == firstPage())
        #expect(model.query.request.base.filter?.chains == [.bitcoin])
    }

    @Test
    func doneAppliesTheSelectionWithoutConfirming() {
        let model = TransactionsFilterSceneViewModel.mock()

        let confirmed = model.onFinishChainsSelection(SelectionResult(items: [.ethereum], isConfirmed: false))

        #expect(confirmed == false)
        #expect(model.networksModel.selectedItems == [.ethereum])
        #expect(model.query.request.base.filter?.chains == [.ethereum])
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
