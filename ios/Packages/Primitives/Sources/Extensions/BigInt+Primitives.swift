import BigInt
import Foundation

public extension BigInt {
    var zero: BigInt {
        BigInt(0)
    }

    var asInt: Int {
        Int(self)
    }

    var asInt64: Int64 {
        Int64(self)
    }

    var asUInt: UInt64 {
        UInt64(self)
    }

    var asUInt32: UInt32 {
        UInt32(self)
    }

    func increase(byPercent percent: Int) -> BigInt {
        let multiplier = 100 + percent
        return self * BigInt(multiplier) / 100
    }

    func decrease(byPercent percent: Int) -> BigInt {
        let multiplier = 100 - percent
        return self * BigInt(multiplier) / 100
    }

    func multiply(byPercent percent: Int) -> BigInt {
        self * BigInt(percent) / 100
    }

    var hexString: String {
        String(self, radix: 16)
    }

    // little endian byte order
    func littleEndianOrder(bytes: Int) -> Data {
        // BigInt.serialize() returns a big endian array, so reverse it for little endian
        var byteArray = Array(serialize().reversed())
        // Ensure the array is bytes long, padding with 0s if necessary
        while byteArray.count < bytes {
            byteArray.append(0)
        }
        return Data(byteArray)
    }

    func isBetween(_ lowerBound: BigInt, and upperBound: BigInt) -> Bool {
        self >= lowerBound && self <= upperBound
    }

    // 256 bit
    static let MAX_256 = BigInt(hex: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")!
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

    static func fromString(_ string: String) -> BigInt {
        if let value = try? BigInt.from(string: string) {
            return value
        }
        return .zero
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
