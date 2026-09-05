// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Gemstone
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct PaymentTransferTests {
    let paymentService = GemPaymentService.mock()

    @Test
    func transactionUsesDecodedTransfer() throws {
        let asset = Asset.mockSolanaUSDC()
        let recipient = "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n"
        let transaction = try Self.paymentTransaction(
            memo: "payment-memo",
            request: Primitives.PaymentRequest(
                address: recipient,
                amount: .atomicValue("19000000"),
                memo: "payment-memo",
                references: nil,
                assetId: asset.id,
            ),
        )

        let destination = PaymentDestinationBuilder.build(transaction: transaction, asset: asset, paymentService: paymentService)
        guard case let .confirm(data) = destination else {
            Issue.record("Expected confirmation")
            return
        }

        #expect(data.inputType.asset == asset)
        #expect(data.value == "19000000")
        #expect(data.recipient.address == recipient)
        #expect(data.recipient.memo == "payment-memo")
        #expect(data.encodedTransaction == "encoded-transaction")
    }

    @Test
    func transactionWithoutMemoConfirms() throws {
        let asset = Asset.mockSolanaUSDC()
        let recipient = "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n"
        let transaction = try Self.paymentTransaction(
            memo: nil,
            request: Primitives.PaymentRequest(
                address: recipient,
                amount: .atomicValue("19000000"),
                memo: nil,
                references: nil,
                assetId: asset.id,
            ),
        )

        let destination = PaymentDestinationBuilder.build(transaction: transaction, asset: asset, paymentService: paymentService)
        guard case let .confirm(data) = destination else {
            Issue.record("Expected confirmation for a decoded transfer without a memo")
            return
        }

        #expect(data.value == "19000000")
        #expect(data.recipient.address == recipient)
        #expect(data.recipient.memo == nil)
        #expect(data.encodedTransaction == "encoded-transaction")
    }

    @Test
    func transactionWithMismatchedAssetFallsBack() throws {
        let asset = Asset.mockSolanaUSDC()
        let transaction = try Self.paymentTransaction(
            memo: "payment-memo",
            request: Primitives.PaymentRequest(
                address: "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n",
                amount: .atomicValue("19000000"),
                memo: "payment-memo",
                references: nil,
                assetId: Primitives.Chain.solana.assetId,
            ),
        )

        let destination = PaymentDestinationBuilder.build(transaction: transaction, asset: asset, paymentService: paymentService)
        guard case let .confirm(data) = destination else {
            Issue.record("Expected confirmation via the encoded transaction fallback")
            return
        }

        #expect(data.value == "0")
        #expect(data.recipient.address.isEmpty)
        #expect(data.recipient.memo == "payment-memo")
        #expect(data.encodedTransaction == "encoded-transaction")
    }

    @Test
    func destinationWithExactAmountConfirms() throws {
        let asset = Asset.mockEthereum()
        let payment = PaymentRequest.mock(address: " \n0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a\r ", amount: .exactValue("1.234"))

        guard case let .confirm(data) = try PaymentDestinationBuilder.transfer(payment: payment, asset: asset, paymentService: paymentService) else {
            Issue.record("Expected confirmation")
            return
        }
        #expect(data.recipient.address == "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a")
        #expect(data.value == "1234000000000000000")
    }

    @Test
    func destinationWithoutMemoRequiresRecipient() throws {
        let xrp = Asset.mock(id: .mock(Chain.xrp), name: "XRP", symbol: "XRP", decimals: 6)
        let payment = PaymentRequest.mock(address: Self.xrpAddress, amount: .exactValue("10"), references: ["reference"], assetId: xrp.id)

        guard case let .recipient(data) = try PaymentDestinationBuilder.transfer(payment: payment, asset: xrp, paymentService: paymentService) else {
            Issue.record("Expected recipient review for XRP payment without a destination tag")
            return
        }
        #expect(data.recipient.address == Self.xrpAddress)
        #expect(data.recipient.memo == nil)
        #expect(data.recipient.references == ["reference"])
        #expect(data.amount == "10")
    }

    private static let xrpAddress = "rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh"

    private static func paymentTransaction(memo: String?, request: Primitives.PaymentRequest?) throws -> GemPaymentTransaction {
        try GemPaymentTransaction(
            merchant: Primitives.ApplicationMetadata(
                name: "Merchant",
                description: "Payment",
                url: "https://example.com",
                icon: "https://example.com/icon.png",
                source: .payment,
            ).map(),
            account: Primitives.ChainAddress(chain: .solana, address: "account").map(),
            transaction: "encoded-transaction",
            transactionType: Primitives.TransactionType.transfer.map(),
            memo: memo,
            request: request.map { $0.json() },
        )
    }
}

private extension GemTransferData {
    var encodedTransaction: String? {
        guard case let .generic(_, _, extra) = inputType, let data = extra.data else { return nil }
        return String(decoding: data, as: UTF8.self)
    }
}
