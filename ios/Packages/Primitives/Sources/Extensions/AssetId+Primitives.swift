import Foundation

extension AssetId: Identifiable {
    public var id: String {
        identifier
    }
}

public extension AssetId {
    static let subTokenSeparator = "::"

    static func from(id: String) throws -> AssetId {
        guard let (chain, tokenId) = AssetId.getData(id: id) else {
            throw AnyError("invalid asset id: \(id)")
        }
        return AssetId(chain: chain, tokenId: tokenId)
    }

    init(chain: Chain) {
        self = .init(chain: chain, tokenId: .none)
    }

    static func getData(id: String) -> (Chain, String?)? {
        let split = id.split(separator: "_")
        if split.count == 1 {
            if let chain = Chain(rawValue: id) {
                return (chain, .none)
            }
        } else if let underscoreIndex = id.firstIndex(of: "_") {
            let chain = String(id[..<underscoreIndex])
            let tokenId = String(id[id.index(after: underscoreIndex)...])
            if let chain = Chain(rawValue: chain) {
                return (chain, tokenId)
            }
        }
        return .none
    }

    var type: AssetSubtype {
        guard let tokenId, !tokenId.isEmpty else {
            return .native
        }
        return .token
    }

    var identifier: String {
        switch type {
        case .native:
            String(format: "%@", chain.rawValue)
        case .token:
            String(format: "%@_%@", chain.rawValue, tokenId ?? "")
        }
    }
}
