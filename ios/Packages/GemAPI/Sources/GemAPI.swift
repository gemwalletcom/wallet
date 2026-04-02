// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SwiftHTTPClient

public enum GemAPI: TargetType {
    case getSwapAssets
    case getConfig
    case getPrices(AssetPricesRequest)
    case getCharts(AssetId, period: String)
    case getAsset(AssetId)
    case getAssets([AssetId], currency: String?)
    case getSearchAssets(query: String, chains: [Chain], tags: [AssetTag])
    case getSearch(query: String, chains: [Chain], tags: [AssetTag])
    case markets

    public var baseUrl: URL {
        Constants.apiURL
    }

    public var method: HTTPMethod {
        switch self {
        case .getSwapAssets,
             .getConfig,
             .getCharts,
             .getAsset,
             .getSearchAssets,
             .getSearch,
             .markets:
            .GET
        case .getAssets,
             .getPrices:
            .POST
        }
    }

    public var path: String {
        switch self {
        case .getSwapAssets:
            return "/v1/swap/assets"
        case .getConfig:
            return "/v1/config"
        case let .getCharts(assetId, _):
            return "/v1/charts/\(assetId.identifier)"
        case let .getAsset(id):
            return "/v1/assets/\(id.identifier.replacingOccurrences(of: "/", with: "%2F"))"
        case let .getAssets(_, currency):
            var path = "/v1/assets"
            if let currency {
                path += "?currency=\(currency)"
            }
            return path
        case .getSearchAssets:
            return "/v1/assets/search"
        case .getSearch:
            return "/v1/search"
        case .getPrices:
            return "/v1/prices"
        case .markets:
            return "/v1/markets"
        }
    }

    public var data: RequestData {
        switch self {
        case .getSwapAssets,
             .getConfig,
             .getAsset,
             .markets:
            .plain
        case let .getAssets(value, _):
            .encodable(value.map(\.identifier))
        case let .getCharts(_, period):
            .params([
                "period": period,
            ])
        case let .getPrices(request):
            .encodable(request)
        case let .getSearchAssets(query, chains, tags),
             let .getSearch(query, chains, tags):
            .params([
                "query": query,
                "chains": chains.map(\.rawValue).joined(separator: ","),
                "tags": tags.map(\.rawValue).joined(separator: ","),
            ])
        }
    }
}

extension Encodable {
    var dictionary: [String: Any]? {
        guard let data = try? JSONEncoder().encode(self) else { return nil }
        return (try? JSONSerialization.jsonObject(with: data, options: [])).flatMap { $0 as? [String: Any] }
    }
}
