// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum AnyCodableValue: Codable, Equatable, Hashable, Sendable {
    case null
    case bool(Bool)
    case int(Int)
    case double(Double)
    case string(String)
    case array([AnyCodableValue])
    case object([String: AnyCodableValue])

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        if container.decodeNil() {
            self = .null
        } else if let bool = try? container.decode(Bool.self) {
            self = .bool(bool)
        } else if let int = try? container.decode(Int.self) {
            self = .int(int)
        } else if let double = try? container.decode(Double.self) {
            self = .double(double)
        } else if let string = try? container.decode(String.self) {
            self = .string(string)
        } else if let array = try? container.decode([AnyCodableValue].self) {
            self = .array(array)
        } else if let object = try? container.decode([String: AnyCodableValue].self) {
            self = .object(object)
        } else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Unable to decode AnyCodableValue",
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        switch self {
        case .null:
            try container.encodeNil()
        case let .bool(value):
            try container.encode(value)
        case let .int(value):
            try container.encode(value)
        case let .double(value):
            try container.encode(value)
        case let .string(value):
            try container.encode(value)
        case let .array(value):
            try container.encode(value)
        case let .object(value):
            try container.encode(value)
        }
    }
}

public extension AnyCodableValue {
    func decode<T: Decodable>(_: T.Type) -> T? {
        // Try direct decode first
        if let data = try? JSONEncoder().encode(self),
           let result = try? JSONDecoder().decode(T.self, from: data)
        {
            return result
        }
        // If value is a string containing JSON, try to parse it
        if case let .string(jsonString) = self,
           let data = jsonString.data(using: .utf8)
        {
            return try? JSONDecoder().decode(T.self, from: data)
        }
        return nil
    }

    static func encode(_ value: some Encodable) -> AnyCodableValue? {
        guard let data = try? JSONEncoder().encode(value) else { return nil }
        return try? JSONDecoder().decode(AnyCodableValue.self, from: data)
    }
}
