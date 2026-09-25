// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemFiatTransactionBadge
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Style
import Testing

struct FiatTransactionViewModelTests {
    @Test
    func theRowComesFromCore() {
        let model = FiatTransactionViewModel.models(
            [.mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18), status: .pending, fiatAmount: 25, value: "1500000000000000000")],
            locale: Locale(identifier: "en_US"),
        )[0]

        let row = model.listItemModel

        #expect(row.titleExtra == "Ethereum (MoonPay)")
        #expect(row.subtitle == "1.5 ETH")
        #expect(row.subtitleExtra == "$25.00")
        #expect(row.titleTag == GemFiatTransactionBadge.pending.text)
    }

    @Test
    func aFailedTransactionIsDimmed() {
        let model = FiatTransactionViewModel.models([.mock(status: .failed)], locale: Locale(identifier: "en_US"))[0]

        #expect(model.listItemModel.subtitleStyle.color == Colors.gray)
    }
}
