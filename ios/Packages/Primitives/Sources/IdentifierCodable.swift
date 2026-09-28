// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol IdentifierCodable: Codable {
    var identifier: String { get }
    static func from(id: String) throws -> Self
}

public extension IdentifierCodable {
    init(core id: String) {
        do {
            self = try Self.from(id: id)
        } catch {
            preconditionFailure("failed to decode \(Self.self) from Core: \(id)")
        }
    }

    init(from decoder: Decoder) throws {
        self = try Self.from(id: decoder.singleValueContainer().decode(String.self))
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(identifier)
    }
}
