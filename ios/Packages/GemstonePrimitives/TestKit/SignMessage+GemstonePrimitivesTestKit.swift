// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.SignMessage
import enum Gemstone.SignDigestType
import Foundation

public extension SignMessage {
    static func mock(
        chain: String = "ethereum",
        signType: SignDigestType = .eip191,
        data: Data = Data("test".utf8),
    ) -> SignMessage {
        SignMessage(chain: chain, signType: signType, data: data)
    }

    static func mockPermitBatch() -> SignMessage {
        .mock(signType: .eip712, data: Data("""
        {
          "types": {
            "EIP712Domain": [
              { "name": "name", "type": "string" },
              { "name": "chainId", "type": "uint256" },
              { "name": "verifyingContract", "type": "address" }
            ],
            "PermitBatch": [
              { "name": "details", "type": "PermitDetails[]" },
              { "name": "spender", "type": "address" },
              { "name": "sigDeadline", "type": "uint256" }
            ],
            "PermitDetails": [
              { "name": "token", "type": "address" },
              { "name": "amount", "type": "uint160" },
              { "name": "expiration", "type": "uint48" },
              { "name": "nonce", "type": "uint48" }
            ]
          },
          "primaryType": "PermitBatch",
          "domain": {
            "name": "Permit2",
            "chainId": "1",
            "verifyingContract": "0x000000000022D473030F116dDEE9F6B43aC78BA3"
          },
          "message": {
            "details": [
              {
                "token": "0x1111111111111111111111111111111111111111",
                "amount": "1000000000000000000",
                "expiration": "1712600000",
                "nonce": "0"
              }
            ],
            "spender": "0x3333333333333333333333333333333333333333",
            "sigDeadline": "1712600500"
          }
        }
        """.utf8))
    }
}
