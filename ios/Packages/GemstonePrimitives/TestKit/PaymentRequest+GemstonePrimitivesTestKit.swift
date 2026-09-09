// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.PaymentAmount
import struct Gemstone.PaymentRequest
import Primitives

public extension PaymentRequest {
    static func mock(
        address: String = "0xcB3028d6120802148f03d6c884D6AD6A210Df62A",
        amount: PaymentAmount? = nil,
        memo: String? = nil,
        label: String? = nil,
        references: [String]? = nil,
        assetId: Primitives.AssetId? = nil,
    ) -> PaymentRequest {
        PaymentRequest(
            address: address,
            amount: amount,
            memo: memo,
            label: label,
            references: references,
            assetId: assetId?.identifier,
        )
    }
}
