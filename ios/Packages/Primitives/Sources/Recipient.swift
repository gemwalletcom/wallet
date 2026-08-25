import Foundation

public struct Recipient: Codable, Equatable, Hashable, Sendable {
    public let name: String?
    public let address: String
    public let memo: String?
    public let references: [String]

    public init(name: String?, address: String, memo: String?, references: [String] = []) {
        self.name = name
        self.address = address
        self.memo = memo
        self.references = references
    }
}

public enum RecipientAssetType: Codable, Equatable, Hashable, Sendable {
    case asset(Asset)
    case nft(NFTAsset)
}

extension RecipientAssetType: Identifiable {
    public var id: String {
        switch self {
        case let .asset(asset): asset.id.identifier
        case let .nft(asset): asset.id.identifier
        }
    }
}

public struct RecipientData: Codable, Equatable, Hashable, Sendable {
    public let recipient: Recipient
    public let amount: String?

    public init(
        recipient: Recipient,
        amount: String?,
    ) {
        self.recipient = recipient
        self.amount = amount
    }
}
