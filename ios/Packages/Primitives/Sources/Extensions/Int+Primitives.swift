// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public extension Int32 {
    var asInt: Int {
        Int(self)
    }

    var asBigInt: BigInt {
        BigInt(self)
    }

    var asString: String {
        String(self)
    }
}

public enum RoundingMode {
    case up
    case down
    case nearest
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

    func roundToNearest(multipleOf base: Int, mode: RoundingMode) -> Int {
        guard base > 0 else { return base }
        switch mode {
        case .up:
            return ((self + base - 1) / base) * base
        case .down:
            return (self / base) * base
        case .nearest:
            return ((self + base / 2) / base) * base
        }
    }

    var asInt32: Int32 {
        Int32(self)
    }

    var asUInt32: UInt32 {
        UInt32(self)
    }

    var asUInt64: UInt64 {
        UInt64(self)
    }

    var asString: String {
        String(self)
    }

    var asBigInt: BigInt {
        BigInt(self)
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

    var asBigInt: BigInt {
        BigInt(self)
    }
}
