// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

struct AssetDetailsInfoViewModel {
    private let priceData: PriceData
    private let market: AssetMarketViewModel?

    private let contractExplorerLink: BlockExplorerLink?

    init(
        priceData: PriceData,
        currency: String,
        contractExplorerLink: BlockExplorerLink?,
    ) {
        self.priceData = priceData
        self.contractExplorerLink = contractExplorerLink
        market = priceData.market.map {
            AssetMarketViewModel(market: $0, assetSymbol: priceData.asset.symbol, currency: currency)
        }
    }

    var marketValues: [MarketValueViewModel] {
        guard let market else { return [] }
        return [market.marketCap, market.fdv, market.tradingVolume].withValues()
    }

    var contractValues: [MarketValueViewModel] {
        [contractViewModel].withValues()
    }

    var supplyValues: [MarketValueViewModel] {
        guard let market else { return [] }
        return [market.circulatingSupply, market.totalSupply, market.maxSupply].withValues()
    }

    var allTimeValues: [MarketValueViewModel] {
        guard let market else { return [] }
        return [market.allTimeHigh, market.allTimeLow].withValues()
    }

    var showLinks: Bool {
        !priceData.links.isEmpty
    }

    var linksViewModel: SocialLinksViewModel {
        SocialLinksViewModel(assetLinks: priceData.links)
    }

    // MARK: - Contract

    private var contractViewModel: MarketValueViewModel {
        MarketValueViewModel(
            title: Localized.Asset.contract,
            subtitle: contractText,
            action: contract.flatMap { contract in
                contractExplorerLink.map {
                    MarketValueViewModel.Action.explorer(
                        ExplorerContextData(copyValue: .address(value: contract, chain: priceData.asset.chain), explorerLink: $0),
                    )
                }
            } ?? .none,
        )
    }

    private var contract: String? {
        try? priceData.asset.getTokenId()
    }

    private var contractText: String? {
        contract.map { AddressFormatter(address: $0, chain: priceData.asset.chain).value() }
    }
}
