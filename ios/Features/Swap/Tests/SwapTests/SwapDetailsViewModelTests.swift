// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemDurationPart
import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemListRow
import struct Gemstone.SwapperQuote
import GemstonePrimitives
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Swap
import SwapTestKit
import Testing

@MainActor
struct SwapDetailsViewModelTests {
    @Test
    func theEstimatedTimeRowAppearsOnlyWhenTheQuoteGivesOne() {
        #expect(durationParts(SwapperQuote.mock(etaInSeconds: nil)) == nil)
        #expect(durationParts(SwapperQuote.mock(etaInSeconds: 30))?.isEmpty == false)
        #expect(durationParts(SwapperQuote.mock(etaInSeconds: 180))?.isEmpty == false)
    }

    @Test
    func switchRate() {
        let model = SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(toValue: 250_000_000_000).swapQuote)

        #expect(model.rateText == "1 ETH ≈ 250,000.00 USDT")

        model.switchRateDirection()
        #expect(model.rateText == "1 USDT ≈ 0.000004 ETH")
    }

    @Test
    func minReceiveAppliesSlippageBasisPoints() {
        let model = SwapDetailsViewModel.mock(selectedQuote: SwapperQuote.mock(toValue: 250_000_000_000).swapQuote)
        let minReceive = model.detailRows.compactMap { row -> GemFormattedNumber? in
            guard case let .amount(title, amount, _) = row, title == .minimumReceive else { return nil }
            return amount
        }.first

        #expect(minReceive?.text() == "248,750 USDT")
    }

    private func durationParts(_ quote: SwapperQuote) -> [GemDurationPart]? {
        SwapDetailsViewModel.mock(selectedQuote: quote.swapQuote).detailRows.compactMap { row -> [GemDurationPart]? in
            guard case let .duration(title, parts, _) = row, title == .estimatedTime else { return nil }
            return parts
        }.first
    }
}
