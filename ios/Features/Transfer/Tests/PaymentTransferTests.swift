// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer

struct PaymentTransferTests {
    let paymentService = GemPaymentService.mock()

    @Test
    func destinationWithExactAmountConfirms() throws {
        let asset = Asset.mockEthereum()
        let payment = PaymentRequest.mock(address: " \n0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a\r ", amount: .exactValue(value: "1.234"))

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
        let payment = PaymentRequest.mock(address: Self.xrpAddress, amount: .exactValue(value: "10"), references: ["reference"], assetId: xrp.id)

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
}
