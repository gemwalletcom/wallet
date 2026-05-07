// Copyright (c). Gem Wallet. All rights reserved.

public extension Chain {
    init(id: String) throws {
        if let chain = Chain(rawValue: id) {
            self = chain
        } else {
            throw AnyError("invalid chain id: \(id)")
        }
    }

    var assetId: AssetId {
        AssetId(chain: self, tokenId: .none)
    }
}

extension Chain: Identifiable {
    public var id: String {
        rawValue
    }
}

public extension Chain {
    var stakeChain: StakeChain? {
        StakeChain(rawValue: rawValue)
    }
}

extension Chain: Comparable {
    public static func < (lhs: Chain, rhs: Chain) -> Bool {
        lhs.rawValue < rhs.rawValue
    }
}
