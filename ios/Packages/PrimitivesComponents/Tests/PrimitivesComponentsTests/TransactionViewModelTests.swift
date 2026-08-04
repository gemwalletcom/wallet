// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct TransactionViewModelTests {
    @Test
    func smartContractCallTitle() {
        let payment = TransactionPaymentMetadata(
            paymentId: "pay_1",
            merchant: PaymentMerchant(name: "Gem Wallet Test Merchant", iconUrl: .none),
            provider: .walletConnectPay,
        )

        #expect(model(metadata: AnyCodableValue.encode(payment)).titleTextValue.text == Localized.Transfer.paymentTitle)
        #expect(model(metadata: .none).titleTextValue.text == Localized.Transfer.SmartContract.title)
    }

    private func model(metadata: AnyCodableValue?) -> TransactionViewModel {
        TransactionViewModel(
            explorerService: MockExplorerLink(),
            transaction: .mock(transaction: .mock(type: .smartContractCall, metadata: metadata)),
            currency: "USD",
        )
    }
}
