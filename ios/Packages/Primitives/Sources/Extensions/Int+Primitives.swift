// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension Int32 {
    var asInt: Int {
        Int(self)
    }

    var asString: String {
        String(self)
    }
}

public extension Int {
    static func from(string: String) throws -> Self {
        guard let value = Self(string) else {
            throw AnyError("invalid int")
        }
        return value
    }

    func isBetween(_ lowerBound: Int, and upperBound: Int) -> Bool {
        self >= lowerBound && self <= upperBound
    }

    var asInt32: Int32 {
        Int32(self)
    }

    var asString: String {
        String(self)
    }

}

public extension Int32 {
    init(string: String) throws {
        guard let value = Int32(string) else {
            throw AnyError("Invalid value: \(string)")
        }
        self = value
    }
}

public extension UInt64 {
    init(string: String) throws {
        guard let value = UInt64(string) else {
            throw AnyError("Invalid value: \(string)")
        }
        self = value
    }

}
