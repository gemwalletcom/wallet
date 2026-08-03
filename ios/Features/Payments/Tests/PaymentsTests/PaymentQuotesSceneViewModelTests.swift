// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import PaymentService
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing
@testable import Payments

@MainActor
struct PaymentQuotesSceneViewModelTests {
    @Test
    func preselectsTheFirstQuote() {
        let first = PaymentQuote.mock(id: "option_1")
        let model = Self.model(quotes: [first, .mock(id: "option_2")])

        model.onAppear()

        #expect(model.selectedItem?.id == first.id)
        #expect(!model.isButtonDisabled)
    }

    @Test
    func selectingAQuoteClosesThePicker() {
        let second = PaymentQuote.mock(id: "option_2")
        let model = Self.model(quotes: [.mock(id: "option_1"), second])
        model.onAppear()
        model.onSelectQuotes()

        model.onFinishQuotesSelection(items: [PaymentQuoteItem(quote: second, formatter: ValueFormatter(style: .short))])

        #expect(model.selectedItem?.id == second.id)
        #expect(!model.isPresentingQuotes)
    }

    @Test
    func confirmIsBlockedOnceThePaymentExpires() async {
        let model = Self.model(quotes: [.mock(id: "option_1")], expiresAt: Date(timeIntervalSinceNow: 0.2))
        model.onAppear()

        #expect(!model.isButtonDisabled)

        await model.awaitExpiry()

        #expect(model.isButtonDisabled)
    }

    private static func model(quotes: [PaymentQuote], expiresAt: Date = Date(timeIntervalSinceNow: 900)) -> PaymentQuotesSceneViewModel {
        PaymentQuotesSceneViewModel(
            request: PaymentQuotesRequest(
                id: "pay_1",
                quotes: .mock(expiresAt: expiresAt, quotes: quotes),
                wallet: .mock(),
                assetsData: [],
                ),
            confirmTransferDelegate: { _ in },
        )
    }
}
