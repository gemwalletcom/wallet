// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

public final class BigNumberFormatter: Sendable {
    public static let standard: BigNumberFormatter = .init(locale: Locale(identifier: "en_US"))

    let locale: Locale
    let decimalSeparator: String

    private let minimumFractionDigits: Int
    private let maximumFractionDigits: Int
    private let groupingSeparator: String

    init(
        locale: Locale = .current,
        minimumFractionDigits: Int = 0,
        maximumFractionDigits: Int = Int.max,
        groupingSeparator: String? = nil,
    ) {
        self.locale = locale
        self.minimumFractionDigits = minimumFractionDigits
        self.maximumFractionDigits = maximumFractionDigits
        decimalSeparator = locale.decimalSeparator ?? "."
        self.groupingSeparator = groupingSeparator ?? (locale.groupingSeparator ?? ",")
    }

    func string(from number: BigInt, decimals: Int) -> String {
        let dividend = BigInt(10).power(decimals)
        let (integerPart, remainder) = number.quotientAndRemainder(dividingBy: dividend)
        let integerString = integerString(from: integerPart)
        let fractionalString = fractionalString(from: BigInt(sign: .plus, magnitude: remainder.magnitude), decimals: decimals)
        if fractionalString.isEmpty {
            return integerString
        }
        return "\(integerString)\(decimalSeparator)\(fractionalString)"
    }

    public func decimal(from number: BigInt, decimals: Int) -> Decimal? {
        let dividend = BigInt(10).power(decimals)
        let (integerPart, remainder) = number.quotientAndRemainder(dividingBy: dividend)
        let integerString = integerPart.description
        let fractionalString = fractionalString(from: BigInt(sign: .plus, magnitude: remainder.magnitude), decimals: decimals)
        if fractionalString.isEmpty {
            return Decimal(string: integerString)
        }
        return Decimal(string: "\(integerString)\(decimalSeparator)\(fractionalString)", locale: locale)
    }

    public func double(from number: BigInt, decimals: Int) -> Double? {
        guard let decimal = decimal(from: number, decimals: decimals) else {
            return .none
        }
        return decimal.doubleValue
    }
}

// MARK: - Private

extension BigNumberFormatter {
    private func integerString(from bigInt: BigInt) -> String {
        var resultString = bigInt.description
        let isNegative = bigInt.sign == .minus
        let endIndex = isNegative ? 1 : 0

        for offset in stride(from: resultString.count - 3, to: endIndex, by: -3) {
            let index = resultString.index(resultString.startIndex, offsetBy: offset)
            resultString.insert(contentsOf: groupingSeparator, at: index)
        }

        return resultString
    }

    private func fractionalString(from number: BigInt, decimals: Int) -> String {
        var number = number
        let digits = number.description.count

        if number == 0 || decimals - digits >= maximumFractionDigits {
            return String(repeating: .zero, count: minimumFractionDigits)
        }

        if decimals < minimumFractionDigits {
            number *= BigInt(10).power(minimumFractionDigits - decimals)
        }
        if decimals > maximumFractionDigits {
            let divisor = BigInt(10).power(decimals - maximumFractionDigits)
            if number > divisor {
                number /= divisor
            }
        }

        var string = number.description
        if digits < decimals {
            string = String(repeating: .zero, count: decimals - digits) + string
        }
        if let lastNonZeroIndex = string.reversed().firstIndex(where: { $0 != "0" })?.base {
            let numberOfZeros = string.distance(from: string.startIndex, to: lastNonZeroIndex)
            if numberOfZeros > minimumFractionDigits {
                let newEndIndex = string.index(string.startIndex, offsetBy: numberOfZeros - minimumFractionDigits)
                string = String(string[string.startIndex ..< newEndIndex])
            }
        }

        return string
    }
}
