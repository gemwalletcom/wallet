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

    var append0x: String {
        if starts(with: "0x") {
            return self
        }
        return "0x" + self
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

    func numberOfLines() -> Int {
        numberOfOccurrencesOf(string: "\n") + 1
    }

    func numberOfOccurrencesOf(string: String) -> Int {
        components(separatedBy: string).count - 1
    }

    func trim() -> String {
        trimmingCharacters(in: .whitespacesAndNewlines)
    }

    func addPadding(number: Int, padding: Character) -> String {
        String(repeatElement(padding, count: number - count)) + self
    }

    func addTrailing(number: Int, padding: Character) -> String {
        self + String(repeatElement(padding, count: number - count))
    }

    func encodedData() throws -> Data {
        guard let data = data(using: .utf8) else {
            throw AnyError("Unable to encode string to data")
        }
        return data
    }

    func base64Encoded() throws -> Data {
        guard let data = Data(base64Encoded: self) else {
            throw AnyError("Unable to base64 encode string to data")
        }
        return data
    }

    func removePrefix(_ prefix: String) -> String {
        if hasPrefix(prefix) {
            return String(dropFirst(prefix.count))
        }
        return self
    }

    func addPrefix(_ prefix: String) -> String {
        if !hasPrefix(prefix) {
            return String(prefix + self)
        }
        return self
    }

    var trimLeadingZeros: String {
        guard let firstNonZeroIndex = firstIndex(where: { $0 != "0" }) else {
            return ""
        }
        return String(self[firstNonZeroIndex...])
    }

    func wrap(_ value: String) -> String {
        "\(value)\(self)\(value)"
    }

    func boldMarkdown() -> String {
        wrap("**")
    }
}

public extension String? {
    var valueOrEmpty: String {
        self ?? .empty
    }
}
