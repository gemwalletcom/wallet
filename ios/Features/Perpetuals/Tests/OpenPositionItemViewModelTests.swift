// Copyright (c). Gem Wallet. All rights reserved.

import Components
@testable import Perpetuals
import PerpetualsTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct OpenPositionItemViewModelTests {
    @Test
    func theRowNamesTheSymbolAndItsDirection() {
        let model = OpenPositionItemViewModel(data: .mock(symbol: "ETH", direction: .long, leverage: 5))

        #expect(model.name == "ETH")
        #expect(model.symbol == nil)

        guard case let .type(value) = model.subtitleView else {
            Issue.record("expected a type subtitle, got \(model.subtitleView)")
            return
        }
        #expect(value.text.isNotEmpty)
    }

    @Test
    func aShortPositionReadsDifferentlyFromALong() {
        let long = OpenPositionItemViewModel(data: .mock(direction: .long))
        let short = OpenPositionItemViewModel(data: .mock(direction: .short))

        guard case let .type(longValue) = long.subtitleView, case let .type(shortValue) = short.subtitleView else {
            Issue.record("expected type subtitles")
            return
        }
        #expect(longValue.text != shortValue.text)
    }

    @Test
    func aZeroSizeLeavesTheBalanceBlank() {
        let sized = OpenPositionItemViewModel(data: .mock(size: 250))
        let empty = OpenPositionItemViewModel(data: .mock(size: 0))

        guard case let .balance(sizedBalance, _) = sized.rightView, case let .balance(emptyBalance, _) = empty.rightView else {
            Issue.record("expected balance right views")
            return
        }
        #expect(sizedBalance.text.contains("250"))
        #expect(emptyBalance.text.isEmpty)
    }
}
