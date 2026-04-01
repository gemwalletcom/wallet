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
    case getAssets([AssetId])
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
            "/v1/swap/assets"
        case .getConfig:
            "/v1/config"
        case let .getCharts(assetId, _):
            "/v1/charts/\(assetId.identifier)"
        case let .getAsset(id):
            "/v1/assets/\(id.identifier.replacingOccurrences(of: "/", with: "%2F"))"
        case .getAssets:
            "/v1/assets"
        case .getSearchAssets:
            "/v1/assets/search"
        case .getSearch:
            "/v1/search"
        case .getPrices:
            "/v1/prices"
        case .markets:
            "/v1/markets"
        }
    }

    public var data: RequestData {
        switch self {
        case .getSwapAssets,
             .getConfig,
             .getAsset,
             .markets:
            .plain
        case let .getAssets(value):
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
