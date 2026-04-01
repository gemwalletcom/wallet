import Foundation
import Primitives
import WalletCore

internal import enum Formatters.MnemonicFormatter

public enum Mnemonic {
    public static func isValidWord(_ word: String) -> Bool {
        WalletCore.Mnemonic.isValidWord(word: word)
    }

    public static func isValidWords(_ words: [String]) -> Bool {
        WalletCore.Mnemonic.isValid(mnemonic: MnemonicFormatter.fromArray(words: words))
    }
}
