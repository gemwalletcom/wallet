//
//  Floating Point Conversion.swift
//  BigInt
//
//  Created by Károly Lőrentey on 2017-08-11.
//  Copyright © 2016-2017 Károly Lőrentey.
//

extension BigUInt {
    public init?<T: BinaryFloatingPoint>(exactly source: T) {
        guard source.isFinite else { return nil }
        guard !source.isZero else { self = 0; return }
        guard source.sign == .plus else { return nil }
        let value = source.rounded(.towardZero)
        guard value == source else { return nil }
        assert(value.floatingPointClass == .positiveNormal)
        assert(value.exponent >= 0)
        let significand = value.significandBitPattern
        self = (BigUInt(1) << value.exponent) + BigUInt(significand) >> (T.significandBitCount - Int(value.exponent))
    }

    public init<T: BinaryFloatingPoint>(_ source: T) {
        self.init(exactly: source.rounded(.towardZero))!
    }
}

extension BigInt {
    public init?<T: BinaryFloatingPoint>(exactly source: T) {
        guard let magnitude = BigUInt(exactly: source.magnitude) else { return nil }
        let sign = BigInt.Sign(source.sign)
        self.init(sign: sign, magnitude: magnitude)
    }

    public init<T: BinaryFloatingPoint>(_ source: T) {
        self.init(exactly: source.rounded(.towardZero))!
    }
}

extension BigInt.Sign {
    public init(_ sign: FloatingPointSign) {
        switch sign {
        case .plus:
            self = .plus
        case .minus:
            self = .minus
        }
    }
}
