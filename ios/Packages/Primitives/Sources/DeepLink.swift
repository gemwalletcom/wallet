// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum DeepLink: Sendable {
    static let host = "gemwallet.com"

    case asset(AssetId)
    case swap(AssetId, AssetId?)
    case perpetuals
    case rewards(code: String?)
    case gift(code: String?)
    case buy(AssetId, amount: Int?)
    case sell(AssetId, amount: Int?)
    case setPriceAlert(AssetId, price: Double?)

    public enum PathComponent: String {
        case tokens
        case swap
        case perpetuals
        case rewards
        case join
        case gift
        case buy
        case sell
        case setPriceAlert
    }

    public var pathComponent: PathComponent {
        switch self {
        case .asset: .tokens
        case .swap: .swap
        case .perpetuals: .perpetuals
        case .rewards: .rewards
        case .gift: .gift
        case .buy: .buy
        case .sell: .sell
        case .setPriceAlert: .setPriceAlert
        }
    }

    public var path: String {
        switch self {
        case let .asset(assetId):
            switch assetId.tokenId {
            case let .some(tokenId): "/\(pathComponent.rawValue)/\(assetId.chain.rawValue)/\(tokenId)"
            case .none: "/\(pathComponent.rawValue)/\(assetId.chain.rawValue)"
            }
        case let .swap(fromAssetId, toAssetId):
            switch toAssetId {
            case let .some(id): "/\(pathComponent.rawValue)/\(fromAssetId.identifier)/\(id.identifier)"
            case .none: "/\(pathComponent.rawValue)/\(fromAssetId.identifier)"
            }
        case .perpetuals: "/\(pathComponent.rawValue)"
        case let .rewards(code):
            switch code {
            case let .some(code): "/\(pathComponent.rawValue)?code=\(code)"
            case .none: "/\(pathComponent.rawValue)"
            }
        case let .gift(code):
            switch code {
            case let .some(code): "/\(pathComponent.rawValue)?code=\(code)"
            case .none: "/\(pathComponent.rawValue)"
            }
        case let .buy(assetId, amount), let .sell(assetId, amount):
            switch amount {
            case let .some(amount): "/\(pathComponent.rawValue)/\(assetId.identifier)?amount=\(amount)"
            case .none: "/\(pathComponent.rawValue)/\(assetId.identifier)"
            }
        case let .setPriceAlert(assetId, price):
            switch price {
            case let .some(price): "/\(pathComponent.rawValue)/\(assetId.identifier)?price=\(price)"
            case .none: "/\(pathComponent.rawValue)/\(assetId.identifier)"
            }
        }
    }

    public var url: URL {
        URL(string: "https://\(Self.host)\(path)")!
    }

    public var localUrl: URL {
        URL(string: "gem://\(path)")!
    }
}
