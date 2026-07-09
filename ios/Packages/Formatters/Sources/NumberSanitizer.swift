// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct NumberSanitizer {
    private let decimalSeparator: Character
    private let maximumFractionDigits: Int?
    private let maximumIntegerDigits: Int?
    private let allowedCharacters: CharacterSet

    public init(
        decimalSeparator: Character = Locale.current.decimalSeparator?.first ?? ".",
        maximumFractionDigits: Int? = nil,
        maximumIntegerDigits: Int? = nil,
        allowedCharacters: CharacterSet = CharacterSet.decimalDigits,
    ) {
        self.decimalSeparator = decimalSeparator
        self.maximumFractionDigits = maximumFractionDigits
        self.maximumIntegerDigits = maximumIntegerDigits
        self.allowedCharacters = allowedCharacters
            .union(CharacterSet(charactersIn: String(decimalSeparator)))
    }

    public func sanitize(_ input: String) -> String {
        let cleanedInput = cleanWhiteSpaceAndSymbols(input)
        let allowedCharacters = filterAllowedCharacters(cleanedInput)
        return sanitizeDecimalSeparator(allowedCharacters)
    }

    // MARK: - Private methods

    private func cleanWhiteSpaceAndSymbols(_ input: String) -> String {
        input.filter { !$0.isWhitespace && !$0.isSymbol }
    }

    private func filterAllowedCharacters(_ input: String) -> String {
        input.filter { $0.unicodeScalars.allSatisfy(allowedCharacters.contains) }
    }

    private func sanitizeDecimalSeparator(_ input: String) -> String {
        guard let separatorIndex = input.firstIndex(of: decimalSeparator) else {
            return limitIntegerDigits(input)
        }

        let integerPart = limitIntegerDigits(String(input.prefix(upTo: separatorIndex)))
        let decimalStartIndex = input.index(after: separatorIndex)
        var decimalPart = input[decimalStartIndex...].filter { $0 != decimalSeparator }
        if let maximumFractionDigits {
            decimalPart = String(decimalPart.prefix(maximumFractionDigits))
        }

        return integerPart + String(decimalSeparator) + decimalPart
    }

    private func limitIntegerDigits(_ integer: String) -> String {
        guard let maximumIntegerDigits else { return integer }
        return String(integer.prefix(maximumIntegerDigits))
    }
}
