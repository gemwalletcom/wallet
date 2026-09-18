// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemFormattedNumber
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Localization
@testable import MarketInsight
@testable import MarketInsightTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct AssetDetailsInfoViewModelTests {
    @Test
    func rowsKeepTheOrderAndTitlesCoreGives() throws {
        let tokenId = try #require(Asset.mockEthereumUSDT().id.tokenId)
        let values = AssetDetailsInfoViewModel.mock().marketValues([
            .marketCap(value: .mock(value: 1), rank: 1),
            .fullyDilutedValuation(value: .mock(value: 2)),
            .tradingVolume(value: .mock(value: 3)),
            .contract(tokenId: tokenId, explorer: nil),
            .maxSupply(value: .mock(value: 21)),
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
        let values = AssetDetailsInfoViewModel.mock().marketValues([.marketCap(value: .mock(value: 1_000_000), rank: 7), .marketCap(value: .mock(value: 1_000_000), rank: nil)])

        #expect(values.map(\.titleTag) == [" #7 ", nil])
    }

    @Test
    func valuesPrintTheNumberCoreFormatted() {
        let supply = GemFormattedNumber.mock(value: 1_500, unit: .symbol(symbol: "USDT"), display: .number(precision: .fraction(min: 0, max: 2)), notation: .plain)
        let values = AssetDetailsInfoViewModel.mock().marketValues([.circulatingSupply(value: supply)])

        #expect(values.map(\.subtitle) == [supply.text()])
    }

    @Test
    func contractOpensTheExplorerOnlyWhenCoreGivesALink() throws {
        let tokenId = try #require(Asset.mockEthereumUSDT().id.tokenId)
        let link = BlockExplorerLink(name: "Etherscan", link: "https://etherscan.io/token/\(tokenId)")
        let values = AssetDetailsInfoViewModel.mock().marketValues([
            .contract(tokenId: tokenId, explorer: link.toGem()),
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
