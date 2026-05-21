// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct PerpetualId: Equatable, Hashable, Sendable {
    public static let separator = "_"

    public let provider: PerpetualProvider
    public let symbol: String

    public init(provider: PerpetualProvider, symbol: String) {
        self.provider = provider
        self.symbol = symbol
    }

    public var identifier: String {
        "\(provider.rawValue)\(Self.separator)\(symbol)"
    }

    public static func from(id: String) throws -> PerpetualId {
        let parts = id.split(separator: Self.separator, maxSplits: 1, omittingEmptySubsequences: false)
        guard parts.count == 2,
              let provider = PerpetualProvider(rawValue: String(parts[0])) else {
            throw AnyError("invalid perpetual id: \(id)")
        }
        return PerpetualId(provider: provider, symbol: String(parts[1]))
    }
}

extension PerpetualId: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        self = try Self.from(id: container.decode(String.self))
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(identifier)
    }
}
