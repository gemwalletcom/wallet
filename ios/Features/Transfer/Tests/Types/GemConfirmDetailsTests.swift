// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmDetails
import enum Gemstone.GemListRow
import func Gemstone.perpetualConfirmDetails
import func Gemstone.swapQuoteDetails
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct GemConfirmDetailsTests {
    @Test
    func swap() {
        let quote = swapQuoteDetails(quote: .mock(), fromAsset: Asset.mock().toGem(), toAsset: Asset.mock().toGem(), fromPrice: nil, toPrice: nil, currency: Currency.usd.toGem())
        guard case let .swapDetails(details) = GemConfirmDetails.swap(details: quote).itemModel else {
            Issue.record("Expected .swapDetails")
            return
        }
        #expect(details == quote)
    }

    @Test
    func perpetual() throws {
        let details = try #require(perpetualConfirmDetails(perpetualType: .open(data: .mock(direction: .long, price: "100", fiatValue: 100, size: "1", slippage: 2, leverage: 3, marketPrice: 100, marginAmount: 33.33))))

        guard case .perpetualDetails = GemConfirmDetails.perpetual(details: details).itemModel else {
            Issue.record("Expected .perpetualDetails")
            return
        }
    }

    @Test
    func perpetualAutoclose() {
        let row = GemListRow.memo(title: .memo, value: "take profit", menu: [])

        guard case let .perpetualModifyPosition(item) = GemConfirmDetails.perpetualAutoclose(row: row).itemModel else {
            Issue.record("Expected .perpetualModifyPosition")
            return
        }
        #expect(item == row)
    }
}
