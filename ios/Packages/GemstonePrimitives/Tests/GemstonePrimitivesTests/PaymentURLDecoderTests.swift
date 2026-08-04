// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

final class PaymentURLDecoderTests {
    @Test
    func testAddress() throws {
        let result = try PaymentURLDecoder.decode("0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326")
        #expect(result == .request(.mock(address: "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326")))
    }

    @Test
    func solana() throws {
        let result1 = try PaymentURLDecoder.decode("HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5")
        #expect(result1 == .request(.mock(address: "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5")))

        let result2 = try PaymentURLDecoder.decode("solana:HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5?amount=0.266232")
        #expect(result2 == .request(.mock(
            address: "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5",
            amount: "0.266232",
            assetId: .mockSolana(),
        )))
    }

    @Test
    func links() throws {
        let solanaPay = "https://api.spherepay.co/v1/public/paymentLink/pay/paymentLink_1"
        let result1 = try PaymentURLDecoder.decode("solana:https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_1")
        #expect(result1 == .link(PaymentLink(provider: .solanaPay, id: solanaPay)))

        let result2 = try PaymentURLDecoder.decode("https://pay.walletconnect.com/?pid=pay_123")
        #expect(result2 == .link(PaymentLink(provider: .walletConnectPay, id: "pay_123")))
    }
}
