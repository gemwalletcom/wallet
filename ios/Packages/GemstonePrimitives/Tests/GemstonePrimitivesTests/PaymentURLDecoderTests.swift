// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstonePrimitives
import Primitives
import Testing

final class PaymentURLDecoderTests {
    @Test
    func decodeRequest() throws {
        #expect(
            try PaymentURLDecoder.decode("0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326")
                == .request(PaymentRequest(
                    address: "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326",
                    amount: .none,
                    memo: .none,
                    assetId: .none,
                )),
        )

        #expect(
            try PaymentURLDecoder.decode("solana:HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5?amount=0.266232&memo=order7")
                == .request(PaymentRequest(
                    address: "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5",
                    amount: .exactValue("0.266232"),
                    memo: "order7",
                    assetId: AssetId(chain: .solana, tokenId: .none),
                )),
        )
    }

    @Test
    func decodeLink() throws {
        #expect(
            try PaymentURLDecoder.decode("solana:https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_1")
                == .link(.solanaPay("https://api.spherepay.co/v1/public/paymentLink/pay/paymentLink_1")),
        )
        #expect(try PaymentURLDecoder.decode("https://pay.walletconnect.com/?pid=pay_123") == .link(.walletConnectPay("pay_123")))
    }

    @Test
    func decodeUnsupported() throws {
        #expect(throws: (any Error).self) {
            try PaymentURLDecoder.decode("https://pay.walletconnect.com/?pid=checkout")
        }
        #expect(throws: (any Error).self) {
            try PaymentURLDecoder.decode("WIFI:S:MyNet;T:WPA;P:secret;;")
        }
    }
}
