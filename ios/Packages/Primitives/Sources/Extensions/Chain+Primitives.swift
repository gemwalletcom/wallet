// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

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

    /// in most cases address is the case, except bitcoin cash
    /// short and full simplifies bitcoincash address
    func shortAddress(address: String) -> String {
        switch self {
        case .bitcoinCash: address.removePrefix("\(rawValue):")
        default: address
        }
    }

    func fullAddress(address: String) -> String {
        switch self {
        case .bitcoinCash: address.addPrefix("\(rawValue):")
        default: address
        }
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
