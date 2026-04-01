// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum SelectAssetType: Identifiable, Hashable, Sendable {
    case send
    case receive(ReceiveAssetType)
    case buy
    case swap(SelectAssetSwapType)
    case manage
    case priceAlert
    case deposit
    case withdraw

    public var id: String {
        switch self {
        case .send: "send"
        case let .receive(type): "receive_\(type.id)"
        case .buy: "buy"
        case let .swap(type): "swap_\(type.id)"
        case .manage: "manage"
        case .priceAlert: "priceAlert"
        case .deposit: "perps"
        case .withdraw: "perps_withdrawal"
        }
    }
}

public extension SelectAssetType {
    func recentActivityData(assetId: AssetId) -> RecentActivityData? {
        switch self {
        case .receive: RecentActivityData(type: .receive, assetId: assetId, toAssetId: nil)
        case .buy: RecentActivityData(type: .fiatBuy, assetId: assetId, toAssetId: nil)
        case .send, .swap, .manage, .priceAlert, .deposit, .withdraw: .none
        }
    }
}

public enum SelectAssetSwapType: Identifiable, Hashable, Sendable {
    case pay
    case receive(chains: [Chain], assetIds: [AssetId])

    public var id: String {
        switch self {
        case .pay: "pay"
        case let .receive(chains, assetIds): "receive_\(chains)_\(assetIds)"
        }
    }
}

public enum ReceiveAssetType: String, Hashable, Identifiable, Sendable {
    case asset
    case collection

    public var id: String {
        rawValue
    }
}
