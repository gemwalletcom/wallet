// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemHeaderAmount
import GemstonePrimitives
import Primitives
import Style

public struct NumericViewModel: Sendable, AmountDisplayable {
    private let header: GemHeaderAmount

    public init(header: GemHeaderAmount) {
        self.header = header
    }

    public var amount: TextValue {
        TextValue(
            text: header.amount.text(),
            style: TextStyle(font: .body, color: header.amount.tone.color, fontWeight: .medium),
            lineLimit: 1,
        )
    }

    public var fiat: TextValue? {
        header.fiat.map {
            TextValue(
                text: $0.text(),
                style: TextStyle(font: .footnote, color: Colors.gray, fontWeight: .medium),
                lineLimit: 1,
            )
        }
    }

    public var assetImage: AssetImage? {
        AssetImage(icon: header.icon)
    }
}
