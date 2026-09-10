// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Localization
@testable import MarketInsight
import Primitives
import PrimitivesTestKit
import Testing

struct AssetDetailsInfoViewModelTests {
    @Test
    func rowsKeepTheOrderAndTitlesCoreGives() throws {
        let tokenId = try #require(Asset.mockEthereumUSDT().id.tokenId)
        let values = AssetDetailsInfoViewModel.mock().marketValues([
            .marketCap(value: 1, rank: 1),
            .fullyDilutedValuation(value: 2),
            .tradingVolume(value: 3),
            .contract(tokenId: tokenId, explorer: nil),
            .maxSupply(value: 21),
        ])

        #expect(values.map(\.title) == [
            Localized.Asset.marketCap,
            Localized.Info.FullyDilutedValuation.title,
            Localized.Asset.tradingVolume,
            Localized.Asset.contract,
            Localized.Info.MaxSupply.title,
        ])
    }

    @Test
    func marketCapCarriesTheRankTagOnlyWhenCoreGivesOne() {
        let values = AssetDetailsInfoViewModel.mock().marketValues([.marketCap(value: 1_000_000, rank: 7), .marketCap(value: 1_000_000, rank: nil)])

        #expect(values.map(\.titleTag) == [" #7 ", nil])
    }

    @Test
    func supplyRowsCarryTheAssetSymbol() {
        let values = AssetDetailsInfoViewModel.mock().marketValues([.circulatingSupply(value: 1_500), .maxSupply(value: 21)])

        #expect(values.map(\.subtitle) == ["1,500.00 USDT", "21.00 USDT"])
    }

    @Test
    func contractOpensTheExplorerOnlyWhenCoreGivesALink() throws {
        let tokenId = try #require(Asset.mockEthereumUSDT().id.tokenId)
        let link = BlockExplorerLink(name: "Etherscan", link: "https://etherscan.io/token/\(tokenId)")
        let values = AssetDetailsInfoViewModel.mock().marketValues([
            .contract(tokenId: tokenId, explorer: link.map()),
            .contract(tokenId: tokenId, explorer: nil),
        ])

        guard case let .explorer(context) = values[0].action else {
            Issue.record("expected an explorer action")
            return
        }
        #expect(context.explorerLink == link)
        guard case .none = values[1].action else {
            Issue.record("expected no action")
            return
        }
    }
}

private extension AssetDetailsInfoViewModel {
    static func mock(asset: Asset = .mockEthereumUSDT(), currency: Currency = .usd) -> AssetDetailsInfoViewModel {
        AssetDetailsInfoViewModel(asset: asset, currency: currency.rawValue)
    }
}
