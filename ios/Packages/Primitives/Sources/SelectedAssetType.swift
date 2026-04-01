// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum SelectedAssetType: Sendable, Hashable, Identifiable {
    case send(RecipientAssetType)
    case receive(ReceiveAssetType)
    case stake(Asset)
    case earn(Asset)
    case buy(Asset, amount: Int?)
    case sell(Asset, amount: Int?)
    case swap(Asset, Asset?)

    public var id: String {
        switch self {
        case let .send(type): "send_\(type.id)"
        case let .receive(type): "receive_\(type.id)"
        case let .stake(asset): "stake_\(asset.id)"
        case let .earn(asset): "earn_\(asset.id)"
        case let .buy(asset, _): "buy_\(asset.id)"
        case let .sell(asset, _): "sell_\(asset.id)"
        case let .swap(fromAsset, toAsset): "swap_\(fromAsset.id)_\(toAsset?.id.identifier ?? "")"
        }
    }
}

public extension SelectedAssetType {
    func recentActivityData(assetId: AssetId) -> RecentActivityData? {
        switch self {
        case .receive: RecentActivityData(type: .receive, assetId: assetId, toAssetId: nil)
        case .buy: RecentActivityData(type: .fiatBuy, assetId: assetId, toAssetId: nil)
        case .sell: RecentActivityData(type: .fiatSell, assetId: assetId, toAssetId: nil)
        case .send, .stake, .earn, .swap: .none
        }
    }
}
