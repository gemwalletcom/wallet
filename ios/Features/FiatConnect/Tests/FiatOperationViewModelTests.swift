// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
@testable import FiatConnect
import protocol Gemstone.GemFiatQuoteServiceProtocol
import GemstonePrimitivesTestKit
import Formatters
import Foundation
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
final class FiatOperationViewModelTests {
    private static func mock(
        service: any GemFiatQuoteServiceProtocol = GemFiatQuoteServiceMock(),
        asset: Asset = .mock(),
    ) -> FiatOperationViewModel {
        FiatOperationViewModel(
            service: service,
            type: .buy,
            asset: asset,
            walletId: .mock(),
            currencyFormatter: CurrencyFormatter(locale: .US, currencyCode: Currency.usd.rawValue),
        )
    }

    @Test
    func shouldSkipFetchWhenAlreadyLoading() {
        let model = FiatOperationViewModelTests.mock()
        model.loadingAmount = 100.0

        #expect(model.shouldSkipFetch(for: 100.0) == true)
        #expect(model.shouldSkipFetch(for: 50.0) == false)
    }

    @Test
    func shouldNotSkipFetchWhenQuotesAlreadyLoadedForSameAmount() {
        let model = FiatOperationViewModelTests.mock()
        model.quotesState = .data(FiatQuotes(amount: 100.0, quotes: [.mock()]))

        #expect(model.shouldSkipFetch(for: 100.0) == false)
    }

    @Test
    func onChangeAmountTextClearsQuoteAndSetsLoading() {
        let model = FiatOperationViewModelTests.mock()
        model.amount = "50"
        model.selectedQuote = .mock()
        model.quotesState = .data(FiatQuotes(amount: 50, quotes: [.mock()]))

        model.onChangeAmountText("", text: "100")

        #expect(model.selectedQuote == nil)
        #expect(model.amount == "100")
        #expect(model.quotesState.isLoading == true)
    }

    @Test
    func onChangeAmountTextPreservesLoadingState() {
        let model = FiatOperationViewModelTests.mock()
        model.amount = "50"
        model.selectedQuote = .mock()
        model.quotesState = .loading

        model.onChangeAmountText("", text: "100")

        #expect(model.selectedQuote == nil)
        #expect(model.amount == "100")
        #expect(model.quotesState.isLoading == true)
    }

    @Test
    func onChangeAmountTextPreservesQuoteWhenAmountUnchanged() {
        let model = FiatOperationViewModelTests.mock()
        let quote = FiatQuote.mock()
        model.amount = "50"
        model.selectedQuote = quote
        model.quotesState = .data(FiatQuotes(amount: 50, quotes: [quote]))

        model.onChangeAmountText("", text: "50")

        #expect(model.selectedQuote == quote)
        #expect(model.amount == "50")
        #expect(model.quotesState.isLoading == false)
    }

    @Test
    func fetchSetsNoDataWhenInputInvalid() {
        let model = FiatOperationViewModelTests.mock()
        model.inputValidationModel.text = "invalid"
        model.quotesState = .loading

        model.load()

        #expect(model.quotesState.isNoData == true)
    }

    @Test
    func fetchSetsNoDataWhenAmountZero() {
        let model = FiatOperationViewModelTests.mock()
        model.inputValidationModel.text = "0"
        model.quotesState = .loading

        model.load()

        #expect(model.quotesState.isNoData == true)
    }

    @Test
    func fetchSetsNoDataWhenValidationFailsWithNoMatchingQuotes() {
        let model = FiatOperationViewModelTests.mock(service: GemFiatQuoteServiceMock(check: { _ in .aboveMaximum(maximum: 10000) }))
        model.inputValidationModel.text = "20000"
        model.quotesState = .loading

        model.load()

        #expect(model.quotesState.isNoData == true)
    }

    @Test
    func fetchPreservesQuotesWhenValidationInvalidForSameAmount() {
        let model = FiatOperationViewModelTests.mock(service: GemFiatQuoteServiceMock(check: { _ in .aboveMaximum(maximum: 10000) }))
        let quote = FiatQuote.mock()
        let quotes = FiatQuotes(amount: 20000.0, quotes: [quote])
        model.quotesState = .data(quotes)
        model.selectedQuote = quote
        model.inputValidationModel.text = "20000"

        model.load()

        #expect(model.quotesState.value?.quotes.count == 1)
        #expect(model.selectedQuote == quote)
    }

    @Test
    func fetchSetsNoDataWhenValidationFailsForDifferentAmount() {
        let model = FiatOperationViewModelTests.mock(service: GemFiatQuoteServiceMock(check: { _ in .aboveMaximum(maximum: 10000) }))
        let quote = FiatQuote.mock()
        let quotes = FiatQuotes(amount: 100.0, quotes: [quote])
        model.quotesState = .data(quotes)
        model.selectedQuote = quote
        model.inputValidationModel.text = "20000"

        model.load()

        #expect(model.quotesState.isNoData == true)
    }
}
