import Foundation

extension NFTAssetId: Identifiable {
    public var id: String {
        identifier
    }
}

public extension NFTAssetId {
    static let tokenIdSeparator = "::"

    init(id: String) throws {
        guard let parsed = NFTAssetId.parse(id: id) else {
            throw AnyError("invalid nft asset id: \(id)")
        }
        self = parsed
    }

    init(asset: NFTAsset) {
        self.init(
            chain: asset.chain,
            contractAddress: asset.contractAddress ?? "",
            tokenId: asset.tokenId,
        )
    }

    var collectionId: NFTCollectionId {
        NFTCollectionId(chain: chain, contractAddress: contractAddress)
    }

    var identifier: String {
        "\(chain.rawValue)_\(contractAddress)\(Self.tokenIdSeparator)\(tokenId)"
    }

    static func parse(id: String) -> NFTAssetId? {
        guard let chainSeparatorIndex = id.firstIndex(of: "_") else {
            return nil
        }
        let chainRaw = String(id[..<chainSeparatorIndex])
        guard let chain = Chain(rawValue: chainRaw) else {
            return nil
        }
        let rest = id[id.index(after: chainSeparatorIndex)...]
        guard let tokenRange = rest.range(of: tokenIdSeparator) else {
            return nil
        }
        let contractAddress = String(rest[..<tokenRange.lowerBound])
        let tokenId = String(rest[tokenRange.upperBound...])
        return NFTAssetId(chain: chain, contractAddress: contractAddress, tokenId: tokenId)
    }
}
