import BigInt
import Foundation

public extension BigInt {
    var zero: BigInt {
        BigInt(0)
    }

    var asInt: Int {
        Int(self)
    }

    func increase(byPercent percent: Int) -> BigInt {
        let multiplier = 100 + percent
        return self * BigInt(multiplier) / 100
    }

    func decrease(byPercent percent: Int) -> BigInt {
        let multiplier = 100 - percent
        return self * BigInt(multiplier) / 100
    }

    func decrease(byBasisPoints basisPoints: Int) -> BigInt {
        let multiplier = 10000 - basisPoints
        return self * BigInt(multiplier) / 10000
    }

    func multiply(byPercent percent: Int) -> BigInt {
        self * BigInt(percent) / 100
    }

    func isBetween(_ lowerBound: BigInt, and upperBound: BigInt) -> Bool {
        self >= lowerBound && self <= upperBound
    }
}

public extension BigInt {
    static func from(string: String) throws -> BigInt {
        if string.isEmpty {
            .zero
        } else if let value = BigInt(string, radix: 10) {
            value
        } else {
            .zero
        }
    }

    static func fromHex(_ hex: String) throws -> BigInt {
        guard let value = BigInt(hex.remove0x, radix: 16) else {
            throw AnyError("invalid hex value: \(hex)")
        }
        return value
    }

    init?(hex: String) {
        if let value = BigInt(hex.remove0x, radix: 16) {
            self = value
        } else {
            return nil
        }
    }
}
