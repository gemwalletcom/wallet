// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAssetMarketRow
import struct Gemstone.GemAssetMarketRows
import Localization
@testable import MarketInsight
import Primitives
import PrimitivesTestKit
import Testing

struct AssetDetailsInfoViewModelTests {
    @Test
    func sectionsFollowTheRowsCoreReturns() throws {
        let tokenId = try #require(Asset.mockEthereumUSDT().id.tokenId)
        let model = AssetDetailsInfoViewModel.mock(
            rows: GemAssetMarketRows(
                market: [.marketCap(value: 1, rank: 1), .fullyDilutedValuation(value: 2), .tradingVolume(value: 3)],
                contract: [.contract(tokenId: tokenId, explorer: nil)],
                supply: [.maxSupply(value: 21)],
                allTime: [],
            ),
        )

        #expect(model.marketValues.map(\.title) == [
            Localized.Asset.marketCap,
            Localized.Info.FullyDilutedValuation.title,
            Localized.Asset.tradingVolume,
        ])
        #expect(model.contractValues.map(\.title) == [Localized.Asset.contract])
        #expect(model.supplyValues.map(\.title) == [Localized.Info.MaxSupply.title])
        #expect(model.allTimeValues.isEmpty)
        #expect(model.showLinks == false)
    }

    @Test
    func marketCapCarriesTheRankTagOnlyWhenCoreGivesOne() {
        let model = AssetDetailsInfoViewModel.mock(rows: GemAssetMarketRows(
            market: [.marketCap(value: 1_000_000, rank: 7), .marketCap(value: 1_000_000, rank: nil)],
            contract: [],
            supply: [],
            allTime: [],
        ))

        #expect(model.marketValues.map(\.titleTag) == [" #7 ", nil])
    }

    @Test
    func supplyRowsCarryTheAssetSymbol() {
        let model = AssetDetailsInfoViewModel.mock(rows: GemAssetMarketRows(
            market: [],
            contract: [],
            supply: [.circulatingSupply(value: 1_500), .maxSupply(value: 21)],
            allTime: [],
        ))

        #expect(model.supplyValues.map(\.subtitle) == ["1,500.00 USDT", "21.00 USDT"])
    }

    @Test
    func contractOpensTheExplorerOnlyWhenCoreGivesALink() throws {
        let tokenId = try #require(Asset.mockEthereumUSDT().id.tokenId)
        let link = BlockExplorerLink(name: "Etherscan", link: "https://etherscan.io/token/\(tokenId)")
        let model = AssetDetailsInfoViewModel.mock(rows: GemAssetMarketRows(
            market: [],
            contract: [.contract(tokenId: tokenId, explorer: link.map()), .contract(tokenId: tokenId, explorer: nil)],
            supply: [],
            allTime: [],
        ))

        guard case let .explorer(context) = model.contractValues[0].action else {
            Issue.record("expected an explorer action")
            return
        }
        #expect(context.explorerLink == link)
        guard case .none = model.contractValues[1].action else {
            Issue.record("expected no action")
            return
        }
    }
}

private extension AssetDetailsInfoViewModel {
    static func mock(
        priceData: PriceData = .mock(asset: .mockEthereumUSDT()),
        rows: GemAssetMarketRows,
        currency: Currency = .usd,
    ) -> AssetDetailsInfoViewModel {
        AssetDetailsInfoViewModel(priceData: priceData, rows: rows, currency: currency.rawValue)
    }
}
