// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Components
import Foundation
import PriceService
import Primitives
import PrimitivesComponents

@MainActor
@Observable
public final class MarketsSceneViewModel: Sendable {
    var state: StateViewType<MarketsViewModel> = .noData

    let service: MarketService
    let assetsService: AssetsService

    public init(
        service: MarketService,
        assetsService: AssetsService,
    ) {
        self.service = service
        self.assetsService = assetsService
    }

    func fetch() async {
        state = .loading
        do {
            let markets = try await service.getMarkets()
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
