import Foundation

public extension String {
    static let zero = "0"
    static let empty = ""

    var asURL: URL? {
        URL(string: self)
    }

    var isNotEmpty: Bool {
        !isEmpty
    }

    var preventingHyphenation: String {
        map { String($0) }.joined(separator: "\u{200B}")
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
