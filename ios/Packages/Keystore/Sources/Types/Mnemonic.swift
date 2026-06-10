import Foundation

internal import Gemstone

public enum Mnemonic {
    private static let gemMnemonic = GemMnemonic()

    public static func generateWords(wordCount: UInt8 = 12) throws -> [String] {
        try gemMnemonic.generate(wordCount: wordCount)
    }

    public static func isValidWord(_ word: String) -> Bool {
        gemMnemonic.isValidWord(word: word)
    }

    public static func isValidWords(_ words: [String]) -> Bool {
        gemMnemonic.isValid(words: words)
    }

    public static func findInvalidWords(_ words: [String]) -> [String] {
        gemMnemonic.findInvalidWords(words: words)
    }

    public static func suggestWords(prefix: String, limit: UInt32? = nil) -> [String] {
        gemMnemonic.suggestWords(prefix: prefix, limit: limit)
    }
}
