// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import protocol Gemstone.GemPriceServiceProtocol
import GemstonePrimitives
import Primitives
import PrimitivesComponents

@MainActor
@Observable
public final class MarketsSceneViewModel: Sendable {
    var state: StateViewType<MarketsViewModel> = .noData

    let service: any GemPriceServiceProtocol
    let assetsService: any GemAssetsServiceProtocol
    private let preferencesService: any GemPreferencesServiceProtocol

    public init(
        service: any GemPriceServiceProtocol,
        assetsService: any GemAssetsServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
    ) {
        self.service = service
        self.assetsService = assetsService
        self.preferencesService = preferencesService
    }

    func fetch() async {
        state = .loading
        do {
            let markets = try await Markets(service.getMarkets())
            let assets = [markets.assets.gainers, markets.assets.losers, markets.assets.trending]
                .compactMap(\.self)
                .flatMap(\.self)

            try await assetsService.prefetchAssets(for: assets)

            state = .data(MarketsViewModel(markets: markets, currencyCode: preferencesService.currencyCode))
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
