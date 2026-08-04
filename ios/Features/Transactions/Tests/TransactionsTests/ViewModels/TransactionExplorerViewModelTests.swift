// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing
@testable import Transactions

struct TransactionExplorerViewModelTests {
    @Test
    func itemModelEmpty_whenPaymentStillCarriesItsPaymentId() {
        let model = makeModel(hash: "pay_1", metadata: paymentMetadata(paymentId: "pay_1"))

        if case .empty = model.itemModel {} else {
            Issue.record("Expected .empty")
        }
    }

    @Test
    func itemModelExplorer_whenPaymentSettledOnChain() {
        let model = makeModel(hash: "0xsettled", metadata: paymentMetadata(paymentId: "pay_1"))

        if case .explorer = model.itemModel {} else {
            Issue.record("Expected .explorer")
        }
    }

    @Test
    func itemModelExplorer_whenTransactionIsNotAPayment() {
        let model = makeModel(hash: "0xhash", metadata: nil)

        if case .explorer = model.itemModel {} else {
            Issue.record("Expected .explorer")
        }
    }

    private func paymentMetadata(paymentId: String) -> AnyCodableValue? {
        AnyCodableValue.encode(TransactionPaymentMetadata(paymentId: paymentId, merchant: .mock(), provider: .walletConnectPay))
    }

    private func makeModel(hash: String, metadata: AnyCodableValue?) -> TransactionExplorerViewModel {
        TransactionExplorerViewModel(
            transactionViewModel: TransactionViewModel(
                explorerService: MockExplorerLink(),
                transaction: .mock(
                    transaction: .mock(hash: hash, metadata: metadata),
                ),
                currency: "USD",
            ),
            explorerService: MockExplorerLink(),
        )
    }
}
