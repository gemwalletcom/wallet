// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol JsonCodable: Codable {}

public extension JsonCodable {
    init(_ json: String) throws {
        self = try JSONDecoder().decode(Self.self, from: Data(json.utf8))
    }

    func jsonString() throws -> String {
        try String(decoding: JSONEncoder().encode(self), as: UTF8.self)
    }
}

extension Primitives.SimulationResult: JsonCodable {}
extension Primitives.SimulationPayloadField: JsonCodable {}
