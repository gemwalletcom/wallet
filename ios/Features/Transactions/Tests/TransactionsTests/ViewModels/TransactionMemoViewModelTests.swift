// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing
@testable import Transactions

struct TransactionMemoViewModelTests {
    @Test
    func itemModelListItem_whenMemoExists() {
        let model = TransactionMemoViewModel(transaction: .mock(assetId: .mockSolana(), memo: "test"))

        if case .listItem = model.itemModel {} else {
            Issue.record("Expected .listItem")
        }
    }

    @Test
    func itemModelEmpty_whenMemoIsEmpty() {
        #expect(matches(TransactionMemoViewModel(transaction: .mock(assetId: .mock(.cosmos), memo: nil))))
        #expect(matches(TransactionMemoViewModel(transaction: .mock(assetId: .mock(.cosmos), memo: ""))))
    }

    private func matches(_ model: TransactionMemoViewModel) -> Bool {
        if case .empty = model.itemModel { return true }
        return false
    }
}
