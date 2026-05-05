// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum PushNotification: Equatable, Sendable {
    case transaction(walletId: WalletId, AssetId, transaction: Transaction)
    case asset(AssetId)
    case walletAsset(WalletId, AssetId)
    case priceAlert(AssetId)
    case buyAsset(AssetId, amount: Int?)
    case swapAsset(AssetId, AssetId?)
    case support
    case rewards
    case stake(WalletId, AssetId)
    case test
    case unknown

    public init(from userInfo: [AnyHashable: Any]) throws {
        guard
            let typeString = userInfo["type"] as? String,
            let type = PushNotificationTypes(rawValue: typeString),
            let dataDict = userInfo["data"] as? [AnyHashable: Any]
        else {
            self = .unknown
            return
        }

        let data = try JSONSerialization.data(withJSONObject: dataDict, options: [])
        let decoder = JSONDateDecoder.standard
        switch type {
        case .transaction:
            let transaction = try decoder.decode(PushNotificationTransaction.self, from: data)
            self = .transaction(walletId: transaction.walletId, transaction.assetId, transaction: transaction.transaction)
        case .asset:
            let asset = try decoder.decode(PushNotificationAsset.self, from: data)
            self = .asset(asset.assetId)
        case .fiatTransaction:
            let value = try decoder.decode(PushNotificationWalletAsset.self, from: data)
            self = .walletAsset(value.walletId, value.assetId)
        case .priceAlert:
            let asset = try decoder.decode(PushNotificationAsset.self, from: data)
            self = .priceAlert(asset.assetId)
        case .buyAsset:
            // TODO: parse amount from push notification data
            let asset = try decoder.decode(PushNotificationAsset.self, from: data)
            self = .buyAsset(asset.assetId, amount: nil)
        case .swapAsset:
            let swapAsset = try decoder.decode(PushNotificationSwapAsset.self, from: data)
            self = .swapAsset(swapAsset.fromAssetId, swapAsset.toAssetId)
        case .support:
            self = .support
        case .rewards:
            self = .rewards
        case .stake:
            let value = try decoder.decode(PushNotificationWalletAsset.self, from: data)
            self = .stake(value.walletId, value.assetId)
        case .test:
            self = .test
        }
    }
}
