// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.fiatTransactionRows
import enum Gemstone.GemFiatTransactionBadge
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Style
import Testing

struct GemFiatTransactionRowTests {
    @Test
    func theRowComesFromCore() throws {
        let row = try #require(fiatTransactionRows(data: [
            FiatTransactionAssetData.mock(
                asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
                transactionType: .buy,
                provider: .moonPay,
                status: .pending,
                fiatAmount: 25,
                fiatCurrency: "USD",
                value: "1500000000000000000",
            ).toGem(),
        ]).first).listItemModel

        #expect(row.titleExtra == "Ethereum (MoonPay)")
        #expect(row.subtitle == "1.5 ETH")
        #expect(row.subtitleExtra == "$25.00")
        #expect(row.titleTag == GemFiatTransactionBadge.pending.text)
    }

    @Test
    func aFailedTransactionIsDimmed() throws {
        let row = try #require(fiatTransactionRows(data: [FiatTransactionAssetData.mock(status: .failed).toGem()]).first)

        #expect(row.listItemModel.subtitleStyle.color == Colors.gray)
    }
}
