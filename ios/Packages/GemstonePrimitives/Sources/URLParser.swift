// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.Deeplink
import func Gemstone.deeplinkDecodeUrl
import func Gemstone.walletConnectDecodeUrl
import enum Gemstone.WalletConnectLink
import Primitives

enum URLParserError: Error {
    case invalidURL(URL)
}

public enum URLParser {
    public static func from(url: URL) throws -> URLAction {
        if let walletConnectLink = walletConnectDecodeUrl(url: url.absoluteString) {
            return .walletConnect(walletConnectLink.map())
        }

        let deeplink: DeepLink = switch deeplinkDecodeUrl(url: url.absoluteString) {
        case let .asset(assetId): try .asset(AssetId(id: assetId))
        case let .swap(fromAssetId, toAssetId): try .swap(AssetId(id: fromAssetId), toAssetId.map { try AssetId(id: $0) })
        case let .rewards(code): .rewards(code: code)
        case let .gift(code): .gift(code: code)
        case let .buy(assetId, amount): try .buy(AssetId(id: assetId), amount: amount.map { Int($0) })
        case let .sell(assetId, amount): try .sell(AssetId(id: assetId), amount: amount.map { Int($0) })
        case let .setPriceAlert(assetId, price): try .setPriceAlert(AssetId(id: assetId), price: price)
        case .perpetuals: .perpetuals
        case .none: throw URLParserError.invalidURL(url)
        }
        return .deeplink(deeplink)
    }
}
