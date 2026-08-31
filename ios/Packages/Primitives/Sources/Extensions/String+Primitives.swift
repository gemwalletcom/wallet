import Foundation

public extension Character {
    static let space: Character = " "
}

public extension String {
    static let zero = "0"
    static let empty = ""

    var remove0x: String {
        if count >= 2, starts(with: "0x") {
            return String(dropFirst(2))
        }
        return self
    }

    var asURL: URL? {
        URL(string: self)
    }

    var isNotEmpty: Bool {
        !isEmpty
    }

    var isEmptyOrZero: Bool {
        isEmpty || self == .zero
    }

    var preventingHyphenation: String {
        map { String($0) }.joined(separator: "\u{200B}")
    }

    func index(from: Int) -> Index {
        index(startIndex, offsetBy: from)
    }

    func truncate(
        first: Int = 6,
        last: Int = 4,
        connector: String = "...",
    ) -> String {
        replacingOccurrences(of: dropFirst(first).dropLast(last), with: connector)
    }

    func numberOfOccurrencesOf(string: String) -> Int {
        components(separatedBy: string).count - 1
    }

    func trim() -> String {
        trimmingCharacters(in: .whitespacesAndNewlines)
    }

    func encodedData() throws -> Data {
        guard let data = data(using: .utf8) else {
            throw AnyError("Unable to encode string to data")
        }
        return data
    }

    func boldMarkdown() -> String {
        "**\(self)**"
    }
}

public extension String? {
    var valueOrEmpty: String {
        self ?? .empty
    }
}
