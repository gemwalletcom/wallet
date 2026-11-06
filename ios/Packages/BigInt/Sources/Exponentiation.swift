//
//  Exponentiation.swift
//  BigInt
//
//  Created by Károly Lőrentey on 2016-01-03.
//  Copyright © 2016-2017 Károly Lőrentey.
//

extension BigUInt {
    //MARK: Exponentiation

    /// Returns this integer raised to the power `exponent`.
    ///
    /// This function calculates the result by [successively squaring the base while halving the exponent][expsqr].
    ///
    /// [expsqr]: https://en.wikipedia.org/wiki/Exponentiation_by_squaring
    ///
    /// - Note: This function can be unreasonably expensive for large exponents, which is why `exponent` is
    ///         a simple integer value.
    /// - Returns: 1 if `exponent == 0`, otherwise `self` raised to `exponent`. (This implies that `0.power(0) == 1`.)
    /// - Complexity: O((exponent * self.count)^log2(3)) or somesuch. The result may require a large amount of memory, too.
    public func power(_ exponent: Int) -> BigUInt {
        if exponent == 0 { return 1 }
        if exponent == 1 { return self }
        if exponent < 0 {
            precondition(!self.isZero)
            return self == 1 ? 1 : 0
        }
        if self <= 1 { return self }
        var result = BigUInt(1)
        var b = self
        var e = exponent
        while e > 0 {
            if e & 1 == 1 {
                result *= b
            }
            e >>= 1
            b *= b
        }
        return result
    }
}

extension BigInt {
    /// Returns this integer raised to the power `exponent`.
    ///
    /// This function calculates the result by [successively squaring the base while halving the exponent][expsqr].
    ///
    /// [expsqr]: https://en.wikipedia.org/wiki/Exponentiation_by_squaring
    ///
    /// - Note: This function can be unreasonably expensive for large exponents, which is why `exponent` is
    ///         a simple integer value.
    /// - Returns: 1 if `exponent == 0`, otherwise `self` raised to `exponent`. (This implies that `0.power(0) == 1`.)
    /// - Complexity: O((exponent * self.count)^log2(3)) or somesuch. The result may require a large amount of memory, too.
    public func power(_ exponent: Int) -> BigInt {
        return BigInt(sign: self.sign == .minus && exponent & 1 != 0 ? .minus : .plus,
                      magnitude: self.magnitude.power(exponent))
    }
}
