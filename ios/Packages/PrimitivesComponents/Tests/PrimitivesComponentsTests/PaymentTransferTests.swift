// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Gemstone
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct PaymentTransferTests {
    @Test
    func transactionUsesDecodedTransfer() throws {
        let asset = Asset.mockSolanaUSDC()
        let recipient = "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n"
        let transaction = Self.paymentTransaction(
            memo: "payment-memo",
            request: GemPaymentRequest(
                address: recipient,
                amount: .atomicValue("19000000"),
                memo: "payment-memo",
                references: nil,
                assetId: asset.id.identifier,
            ),
        )

        let destination = try PaymentDestinationBuilder.build(transaction: transaction, asset: asset)
        guard case let .confirm(data) = destination else {
            Issue.record("Expected confirmation")
            return
        }

        #expect(data.type.asset == asset)
        #expect(data.value == 19_000_000)
        #expect(data.recipientData.recipient.address == recipient)
        #expect(data.recipientData.recipient.memo == "payment-memo")
        #expect(try data.encodedTransaction() == "encoded-transaction")
    }

    @Test
    func transactionWithoutMemoConfirms() throws {
        let asset = Asset.mockSolanaUSDC()
        let recipient = "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n"
        let transaction = Self.paymentTransaction(
            memo: nil,
            request: GemPaymentRequest(
                address: recipient,
                amount: .atomicValue("19000000"),
                memo: nil,
                references: nil,
                assetId: asset.id.identifier,
            ),
        )

        let destination = try PaymentDestinationBuilder.build(transaction: transaction, asset: asset)
        guard case let .confirm(data) = destination else {
            Issue.record("Expected confirmation for a decoded transfer without a memo")
            return
        }

        #expect(data.value == 19_000_000)
        #expect(data.recipientData.recipient.address == recipient)
        #expect(data.recipientData.recipient.memo == nil)
        #expect(try data.encodedTransaction() == "encoded-transaction")
    }

    @Test
    func transactionWithMismatchedAssetFallsBack() throws {
        let asset = Asset.mockSolanaUSDC()
        let transaction = Self.paymentTransaction(
            memo: "payment-memo",
            request: GemPaymentRequest(
                address: "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n",
                amount: .atomicValue("19000000"),
                memo: "payment-memo",
                references: nil,
                assetId: Primitives.Chain.solana.assetId.identifier,
            ),
        )

        let destination = try PaymentDestinationBuilder.build(transaction: transaction, asset: asset)
        guard case let .confirm(data) = destination else {
            Issue.record("Expected confirmation via the encoded transaction fallback")
            return
        }

        #expect(data.value == .zero)
        #expect(data.recipientData.recipient.address.isEmpty)
        #expect(data.recipientData.recipient.memo == "payment-memo")
        #expect(try data.encodedTransaction() == "encoded-transaction")
    }

    @Test
    func destinationWithExactAmountConfirms() throws {
        let asset = Asset.mockEthereum()
        let address = "0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a"
        let checksummedAddress = "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a"
        let payment = PaymentRequest.mock(address: " \n\(address)\r ", amount: .exactValue("1.234"))

        let destination = try PaymentDestinationBuilder.transfer(payment: payment, asset: asset)

        guard case let .confirm(data) = destination else {
            Issue.record("Expected confirmation")
            return
        }
        #expect(data.recipientData.recipient.address == checksummedAddress)
        #expect(data.amount == .exact(BigInt("1234000000000000000")))
    }

    @Test
    func destinationWithoutAmountRequiresRecipient() throws {
        let asset = Asset.mockEthereum()
        let payment = PaymentRequest.mock(address: "0x123", memo: "test memo", references: ["reference"])

        let destination = try PaymentDestinationBuilder.transfer(payment: payment, asset: asset)

        guard case let .recipient(data) = destination else {
            Issue.record("Expected recipient review")
            return
        }
        #expect(data.recipient.address == payment.address)
        #expect(data.recipient.memo == payment.memo)
        #expect(data.recipient.references == ["reference"])
        #expect(data.amount == nil)
    }

    @Test
    func destinationWithAmountAndMemoConfirms() throws {
        let xrp = Asset.mock(id: .mock(Chain.xrp), name: "XRP", symbol: "XRP", decimals: 6)
        let payment = PaymentRequest.mock(address: Self.xrpAddress, amount: .exactValue("10"), memo: "12345", assetId: xrp.id)

        let destination = try PaymentDestinationBuilder.transfer(payment: payment, asset: xrp)

        guard case let .confirm(data) = destination else {
            Issue.record("Expected confirmation for tagged XRP payment")
            return
        }
        #expect(data.recipientData.recipient.address == Self.xrpAddress)
        #expect(data.recipientData.recipient.memo == "12345")
        #expect(data.amount == .exact(BigInt(10_000_000)))
    }

    @Test
    func destinationWithAmountWithoutMemoRequiresRecipient() throws {
        let xrp = Asset.mock(id: .mock(Chain.xrp), name: "XRP", symbol: "XRP", decimals: 6)
        let payment = PaymentRequest.mock(address: Self.xrpAddress, amount: .exactValue("10"), assetId: xrp.id)

        let destination = try PaymentDestinationBuilder.transfer(payment: payment, asset: xrp)

        guard case let .recipient(data) = destination else {
            Issue.record("Expected recipient review for XRP payment without a destination tag")
            return
        }
        #expect(data.recipient.address == Self.xrpAddress)
        #expect(data.recipient.memo == nil)
        #expect(data.amount == "10")
    }

    @Test
    func destinationBelowSmallestUnitRequiresRecipient() throws {
        let asset = Asset.mockEthereum()
        let payment = PaymentRequest.mock(
            address: "0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a",
            amount: .exactValue("0.0000000000000000001"),
        )

        let destination = try PaymentDestinationBuilder.transfer(payment: payment, asset: asset)

        guard case let .recipient(data) = destination else {
            Issue.record("Expected recipient review for an unrepresentable ETH amount")
            return
        }
        #expect(data.amount == "0.0000000000000000001")
    }

    @Test
    func destinationSolanaWithoutMemoConfirms() throws {
        let asset = Asset.mockSolanaUSDC()
        let payment = PaymentRequest.mock(
            address: "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5",
            amount: .exactValue("1"),
            assetId: asset.id,
        )

        let destination = try PaymentDestinationBuilder.transfer(payment: payment, asset: asset)

        guard case let .confirm(data) = destination else {
            Issue.record("Expected a Solana payment without a memo to confirm")
            return
        }
        #expect(data.amount == .exact(BigInt(1_000_000)))
        #expect(data.recipientData.recipient.memo == nil)
    }

    private static let xrpAddress = "rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh"

    private static func paymentTransaction(memo: String?, request: GemPaymentRequest?) -> GemPaymentTransaction {
        GemPaymentTransaction(
            merchant: GemApplicationMetadata(
                name: "Merchant",
                description: "Payment",
                url: "https://example.com",
                icon: "https://example.com/icon.png",
                source: .payment,
            ),
            account: Gemstone.ChainAddress(chain: Primitives.Chain.solana.rawValue, address: "account"),
            transaction: "encoded-transaction",
            transactionType: .transfer,
            memo: memo,
            request: request,
        )
    }
}
