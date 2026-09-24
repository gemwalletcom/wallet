// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

struct PhraseInput: Equatable {
    let text: String
    let cursor: Int

    init(text: String, cursor: Int?) {
        let length = text.utf16.count
        self.text = text
        self.cursor = min(cursor ?? length, length)
    }

    var word: String {
        (text as NSString).substring(with: wordRange)
    }

    var wordCursor: Int {
        cursor - wordRange.location
    }

    func completing(with suggestion: String) -> PhraseInput {
        let range = wordRange
        let rest = (text as NSString).substring(from: NSMaxRange(range))
        let separator = rest.first.flatMap { $0.isWhitespace ? String($0) : nil }
        let head = (text as NSString).substring(to: range.location) + suggestion + (separator ?? " ")
        let tail = separator == nil ? rest : String(rest.dropFirst())
        return PhraseInput(text: head + tail, cursor: head.utf16.count)
    }

    private var wordRange: NSRange {
        let units = text as NSString
        var start = cursor
        while start > 0, !Self.isWhitespace(units.character(at: start - 1)) {
            start -= 1
        }
        var end = cursor
        while end < units.length, !Self.isWhitespace(units.character(at: end)) {
            end += 1
        }
        return NSRange(location: start, length: end - start)
    }

    private static func isWhitespace(_ unit: unichar) -> Bool {
        UnicodeScalar(unit).map { CharacterSet.whitespacesAndNewlines.contains($0) } ?? false
    }
}
