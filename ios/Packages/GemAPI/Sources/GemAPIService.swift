// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SwiftHTTPClient

public protocol GemAPIAssetsService: Sendable {
    func getAssets(currency: String?, assetIds: [AssetId]) async throws -> [AssetBasic]
}

public struct GemAPIService {
    let provider: Provider<GemAPI>

    public static let shared = GemAPIService()
    public static let sharedProvider = Provider<GemAPI>()

    public init(provider: Provider<GemAPI> = Self.sharedProvider) {
        self.provider = provider
    }
}

extension GemAPIService: GemAPIAssetsService {
    public func getAssets(currency: String?, assetIds: [AssetId]) async throws -> [AssetBasic] {
        try await provider
            .request(.getAssets(assetIds, currency: currency))
            .mapResponse(as: [AssetBasic].self)
    }
}

public extension SwiftHTTPClient.Response {
    @discardableResult
    func mapResponse<T: Decodable>(as type: T.Type) throws -> T {
        try mapOrError(as: type, asError: ResponseError.self)
    }
}
