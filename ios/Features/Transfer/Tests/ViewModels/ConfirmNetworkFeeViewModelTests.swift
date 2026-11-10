// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmFeeRow
import struct Gemstone.GemFeeText
import enum Gemstone.GemInfoTopic
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import Testing
@testable import Transfer

struct ConfirmNetworkFeeViewModelTests {
    @Test
    func readyPrintsTheFeeTextCoreBuilt() {
        let text = GemFeeText.mock(value: .mock(value: 0.0001446, unit: .symbol(symbol: "BNB")), extra: .text(text: "$0.11"))
        let model = ConfirmNetworkFeeViewModel(
            feeRow: .mock(title: .networkFee, value: .ready(text: text), link: .enabled),
            onInfo: { _ in },
        )

        guard case let .networkFee(item, link) = model.itemModel else {
            Issue.record("Expected network fee item")
            return
        }
        #expect(item.subtitle == text.value.text())
        #expect(item.subtitleExtra == "$0.11")
        #expect(link == .enabled)
    }

    @Test
    func unavailablePrintsItsTextAndStaysClosed() {
        let model = ConfirmNetworkFeeViewModel(
            feeRow: .mock(title: .networkFee, value: .unavailable(text: "-"), link: .none),
            onInfo: { _ in },
        )

        guard case let .networkFee(item, link) = model.itemModel else {
            Issue.record("Expected network fee item")
            return
        }
        #expect(item.subtitle == "-")
        #expect(item.subtitleExtra == nil)
        #expect(link == .none)
    }
}
