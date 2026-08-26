// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Components
import Foundation
import protocol Gemstone.GemPriceServiceProtocol
import GemstonePrimitives
import Primitives
import PrimitivesComponents

@MainActor
@Observable
public final class MarketsSceneViewModel: Sendable {
    var state: StateViewType<MarketsViewModel> = .noData

    let service: any GemPriceServiceProtocol
    let assetsService: AssetsService

    public init(
        service: any GemPriceServiceProtocol,
        assetsService: AssetsService,
    ) {
        self.service = service
        self.assetsService = assetsService
    }

    func fetch() async {
        state = .loading
        do {
            let markets = try await Markets(service.getMarkets())
            let assets = [markets.assets.gainers, markets.assets.losers, markets.assets.trending]
                .compactMap(\.self)
                .flatMap(\.self)

            try await assetsService.prefetchAssets(assetIds: assets)

            state = .data(MarketsViewModel(markets: markets))
        } catch {
            state = .error(error)
            debugLog("get markets error: \(error)")
        }
    }

    var title: String {
        "Markets"
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .markets)
    }
}
