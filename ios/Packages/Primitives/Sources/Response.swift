import Foundation

public struct ResponseError: Codable, Sendable, LocalizedError {
    public let message: String

    public var errorDescription: String? {
        message
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let errorContainer = try container.nestedContainer(keyedBy: ErrorCodingKeys.self, forKey: .error)
        message = try errorContainer.decode(String.self, forKey: .message)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        var errorContainer = container.nestedContainer(keyedBy: ErrorCodingKeys.self, forKey: .error)
        try errorContainer.encode(message, forKey: .message)
    }

    private enum CodingKeys: String, CodingKey {
        case error
    }

    private enum ErrorCodingKeys: String, CodingKey {
        case message
    }
}
