import Foundation

public struct Response {
    public let body: Data

    public init(body: Data) {
        self.body = body
    }

    public static let standardDecoder = {
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601WithFractionalSeconds
        return decoder
    }()

    public static func make(data: Data, response urlResponse: URLResponse?) throws -> Response {
        guard urlResponse is HTTPURLResponse else {
            throw URLError(.badServerResponse)
        }
        return Response(body: data)
    }

    public func map<T: Decodable>(as type: T.Type, _ decoder: JSONDecoder = Self.standardDecoder) throws -> T {
        try decoder.decode(type, from: body)
    }
}

// same code lives in primitives, allow to inject json / date formatter on init

private extension Formatter {
    nonisolated(unsafe) static let customISO8601DateFormatter: ISO8601DateFormatter = {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return formatter
    }()

    nonisolated(unsafe) static let customISO8601DateFormatterNoSeconds: ISO8601DateFormatter = {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime]
        return formatter
    }()
}

extension JSONDecoder.DateDecodingStrategy {
    static let iso8601WithFractionalSeconds = custom { decoder in
        let dateStr = try decoder.singleValueContainer().decode(String.self)
        if let date = Formatter.customISO8601DateFormatter.date(from: dateStr) {
            return date
        }
        if let date = Formatter.customISO8601DateFormatterNoSeconds.date(from: dateStr) {
            return date
        }
        throw DecodingError.dataCorrupted(DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Invalid date"))
    }
}
