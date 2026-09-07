// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

private enum JsonCodableEncoder {
    static let standard: JSONEncoder = {
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        return encoder
    }()
}

public protocol JsonCodable: Codable {}

public extension JsonCodable {
    init(_ json: String) throws {
        self = try JSONDateDecoder.standard.decode(Self.self, from: Data(json.utf8))
    }

    init(core json: String) {
        do {
            self = try Self(json)
        } catch {
            preconditionFailure("failed to decode \(Self.self) from Core: \(error)")
        }
    }

    func json() -> String {
        guard let data = try? JsonCodableEncoder.standard.encode(self) else {
            assertionFailure("failed to serialize \(Self.self)")
            return ""
        }
        return String(decoding: data, as: UTF8.self)
    }
}
