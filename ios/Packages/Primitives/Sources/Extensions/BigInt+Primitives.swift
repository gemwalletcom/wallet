import BigInt
import Foundation

public extension BigInt {
    var zero: BigInt {
        BigInt(0)
    }

    var asInt: Int {
        Int(self)
    }

    func isBetween(_ lowerBound: BigInt, and upperBound: BigInt) -> Bool {
        self >= lowerBound && self <= upperBound
    }
}

public extension BigInt {
    static func from(string: String) -> BigInt {
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
