// Copyright (c). Gem Wallet. All rights reserved.

import ReownWalletKit

extension AnyCodable {
    static func null() -> AnyCodable {
        AnyCodable(Null())
    }

    private struct Null: Codable {
        func encode(to encoder: Encoder) throws {
            var container = encoder.singleValueContainer()
            try container.encodeNil()
        }
    }
}
