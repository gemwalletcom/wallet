import Foundation

extension NFTCollectionId: Identifiable {
    public var id: String {
        identifier
    }
}

public extension NFTCollectionId {
    init(id: String) throws {
        guard let parsed = NFTCollectionId.parse(id: id) else {
            throw AnyError("invalid nft collection id: \(id)")
        }
        self = parsed
    }

    init(collection: NFTCollection) {
        self.init(chain: collection.chain, contractAddress: collection.contractAddress)
    }

    var identifier: String {
        "\(chain.rawValue)_\(contractAddress)"
    }

    static func parse(id: String) -> NFTCollectionId? {
        guard let separator = id.firstIndex(of: "_") else {
            return nil
        }
        let chainRaw = String(id[..<separator])
        guard let chain = Chain(rawValue: chainRaw) else {
            return nil
        }
        let contractAddress = String(id[id.index(after: separator)...])
        guard !contractAddress.isEmpty else {
            return nil
        }
        return NFTCollectionId(chain: chain, contractAddress: contractAddress)
    }
}
