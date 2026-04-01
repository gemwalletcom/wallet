// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum MnemonicFormatter {
    private static let separator = " "

    public static func fromArray(words: [String]) -> String {
        words.joined(separator: separator)
    }

    public static func toArray(string: String) -> [String] {
        string
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .components(separatedBy: separator)
    }
}
