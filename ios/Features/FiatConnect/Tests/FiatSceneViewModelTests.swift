// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import struct Gemstone.GemBalanceRequirement
import struct Gemstone.GemFiatQuoteRequest
import struct Gemstone.GemFiatQuotesResult
import protocol Gemstone.GemFiatQuoteServiceProtocol
import enum Gemstone.GemServiceError
import GemstoneServicesTestKit
import BigInt
@testable import FiatConnect
import GemstoneServices
import Formatters
import Foundation
import Localization
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing

@MainActor
final class FiatSceneViewModelTests {
    private static func mock(
        service: any GemFiatQuoteServiceProtocol = GemFiatQuoteServiceMock(),
        assetAddress: AssetAddress = .mock(),
        wallet: Wallet = .mock(),
        type: FiatQuoteType = .buy,
        amount: Int? = nil,
    ) -> FiatSceneViewModel {
        FiatSceneViewModel(
            service: service,
            assetAddress: assetAddress,
            wallet: wallet,
            type: type,
            amount: amount,
            locale: .US,
        )
    }

    private static func load(_ model: FiatSceneViewModel, quotes: [FiatQuote], amount: Double = 50, type: FiatQuoteType = .buy, error: GemServiceError? = nil) {
        model.session = model.session.onQuoteResults(
            results: GemFiatQuotesResult(
                request: GemFiatQuoteRequest(quoteType: type.map(), amount: amount),
                quotes: quotes.map { $0.json() },
                error: error,
            ),
        )
    }

    @Test
    func defaultAmountText() {
        let model = FiatSceneViewModelTests.mock()
        #expect(model.inputValidationModel.text == "50")

        model.type = .sell
        model.onChangeType(oldType: .buy, newType: .sell)

        #expect(model.inputValidationModel.text == "100")
    }

    @Test
    func selectBuyAmount() {
        let model = FiatSceneViewModelTests.mock()
        model.onSelect(amount: 150)

        #expect(model.inputValidationModel.text == "150")

        model.onSelect(amount: 1)

        #expect(model.inputValidationModel.text == "1")
    }

    @Test
    func selectSellAmount() {
        let model = FiatSceneViewModelTests.mock()
        model.type = .sell
        model.onChangeType(oldType: .buy, newType: .sell)

        model.onSelect(amount: 50)

        #expect(model.inputValidationModel.text == "50")

        model.onSelect(amount: 100)

        #expect(model.inputValidationModel.text == "100")
    }

    @Test
    func testCurrencySymbol() {
        let model = FiatSceneViewModelTests.mock()
        #expect(model.currencyInputConfig.currencySymbol == "$")

        model.type = .sell
        model.onChangeType(oldType: .buy, newType: .sell)

        #expect(model.currencyInputConfig.currencySymbol == "$")
    }

    @Test
    func buttonsTitle() {
        let model = FiatSceneViewModelTests.mock()

        #expect(model.buttonTitle(amount: 10) == "$10")

        model.type = .sell
        model.onChangeType(oldType: .buy, newType: .sell)

        #expect(model.buttonTitle(amount: 100) == "$100")
    }

    @Test
    func assetBalanceIncludesSymbol() {
        let asset = Asset.mockTron()
        let model = FiatSceneViewModelTests.mock(assetAddress: .mock(asset: asset))

        model.assetQuery.value = .mock(
            asset: asset,
            balance: .mock(available: BigInt(66_670_000)),
        )

        #expect(model.assetBalance == "66.67 TRX")
    }

    @Test
    func showFiatTypePickerWhenSellEnabledWithZeroBalance() {
        let model = FiatSceneViewModelTests.mock()

        model.assetQuery.value = .mock(
            balance: .zero,
            metadata: .mock(isSellEnabled: true),
        )

        #expect(model.showFiatTypePicker)
    }

    @Test
    func unsupportedSellRouteFallsBackToBuyWithAmount() {
        let model = FiatSceneViewModelTests.mock(type: .sell, amount: 40)
        let previousAssetData = model.assetData
        let unsupportedAssetData = AssetData.mock(metadata: .mock(isSellEnabled: false))
        model.assetQuery.value = unsupportedAssetData
        model.onAssetDataChange(previousAssetData, unsupportedAssetData)

        #expect(!model.showFiatTypePicker)
        #expect(model.type == .buy)
        #expect(model.session.sell.amount == "40")
        #expect(model.session.buy.amount == "40")
        #expect(model.inputValidationModel.text == "40")
        #expect(model.loadTrigger == FiatLoadTrigger(type: .buy, amount: "40", isImmediate: true))
        #expect(model.title == Localized.Buy.title(model.asset.name))
    }

    @Test
    func testRateValue() {
        let model = FiatSceneViewModelTests.mock()
        FiatSceneViewModelTests.load(model, quotes: [.mock(fiatAmount: 1200, cryptoAmount: 2.0)])

        #expect(model.rateValue == "1 \(model.asset.symbol) ≈ $600.00")
        #expect(model.cryptoAmountValue == "≈ 2.00 BTC")
    }

    @Test
    func balanceChangeReachesTheSession() {
        let asset = Asset.mockEthereumUSDT()
        let model = FiatSceneViewModelTests.mock(assetAddress: .mock(asset: asset), type: .sell)

        model.onAssetDataChange(
            .mock(asset: asset),
            .mock(asset: asset, balance: .mock(available: BigInt(415_650_000))),
        )

        #expect(model.session.available == 415_650_000)
    }

    @Test
    func selectingProviderRevalidatesSellBalance() {
        let affordable = FiatQuote.mock(fiatAmount: 100, cryptoAmount: 1, type: .sell)
        let unaffordable = FiatQuote.mock(fiatAmount: 100, cryptoAmount: 3, type: .sell, providerId: "transak")
        let service = GemFiatQuoteServiceMock(check: { quote in
            quote?.cryptoAmount == 3 ? .insufficientBalance(requirement: GemBalanceRequirement(required: 300_000_000, available: 200_000_000, shortfall: 100_000_000)) : .valid
        })
        let model = FiatSceneViewModelTests.mock(service: service, type: .sell)

        model.onAssetDataChange(
            .mock(),
            .mock(balance: .mock(available: BigInt(200_000_000))),
        )
        FiatSceneViewModelTests.load(model, quotes: [affordable, unaffordable], amount: 100, type: .sell)

        #expect(model.selectedQuote == affordable)
        #expect(model.allowSelectProvider)
        #expect(model.actionButtonState == .normal)

        model.onSelectQuotes([FiatQuoteViewModel(asset: model.asset, quote: unaffordable, formatter: CurrencyFormatter(locale: .US, currencyCode: Currency.usd.rawValue))])

        #expect(model.selectedQuote == unaffordable)
        #expect(model.inputValidationModel.isInvalid)
        #expect(model.actionButtonState == .disabled)
        #expect(!model.isPresentingFiatProvider)
    }

    @Test
    func actionButtonStateFollowsTheSession() {
        let model = FiatSceneViewModelTests.mock()
        #expect(model.actionButtonState == .loading(showProgress: true))

        FiatSceneViewModelTests.load(model, quotes: [])
        #expect(model.actionButtonState == .disabled)
        #expect(model.emptyTitle == Localized.Buy.noResults)

        model.onChangeAmountText("", text: "0")
        #expect(model.actionButtonState == .disabled)
        #expect(model.emptyTitle == Localized.Input.enterAmountTo(Localized.Wallet.buy))

        model.onChangeAmountText("", text: "100")
        FiatSceneViewModelTests.load(model, quotes: [.mock(fiatAmount: 100, cryptoAmount: 1)], amount: 100)
        #expect(model.actionButtonState == .normal)
        #expect(model.actionButtonTitle == Localized.Common.continue)

        model.urlState = .loading
        #expect(model.actionButtonState == .loading(showProgress: true))
    }

    @Test
    func failedQuotesOfferARetry() {
        let model = FiatSceneViewModelTests.mock()
        FiatSceneViewModelTests.load(model, quotes: [], error: .Api(msg: "offline"))

        #expect(model.quotesState.isError)
        #expect(model.actionButtonTitle == Localized.Common.tryAgain)
        #expect(model.actionButtonState == .normal)
    }

    @Test
    func urlStateInitialValue() {
        let model = FiatSceneViewModelTests.mock()

        #expect(model.urlState.isNoData == true)
        #expect(model.urlState.isLoading == false)
    }

    @Test
    func loadTriggerOnChangeTypeIsImmediate() {
        let model = FiatSceneViewModelTests.mock()

        model.onChangeType(oldType: .buy, newType: .sell)

        #expect(model.loadTrigger.type == .sell)
        #expect(model.loadTrigger.isImmediate == true)
    }

    @Test
    func loadTriggerOnSelectAmountIsImmediate() {
        let model = FiatSceneViewModelTests.mock()

        model.onSelect(amount: 250)

        #expect(model.loadTrigger.amount == "250")
        #expect(model.loadTrigger.isImmediate == true)
    }

    @Test
    func loadTriggerOnChangeAmountTextIsDebounced() {
        let model = FiatSceneViewModelTests.mock()

        model.onChangeAmountText("", text: "123")

        #expect(model.loadTrigger.amount == "123")
        #expect(model.loadTrigger.isImmediate == false)
        #expect(model.session.amount == "123")
    }

    @Test
    func loadTriggerOnSelectRandomAmountIsImmediate() {
        let model = FiatSceneViewModelTests.mock()

        model.onSelectRandomAmount()

        #expect(model.loadTrigger.isImmediate == true)
    }

    @Test
    func presetSelectionDoesNotScheduleSecondDebouncedFetch() {
        let model = FiatSceneViewModelTests.mock()
        FiatSceneViewModelTests.load(model, quotes: [], error: .Api(msg: "offline"))

        model.onSelect(amount: 250)

        #expect(model.session.amount == "250")
        #expect(model.inputValidationModel.text == "250")
        #expect(model.quotesState.isLoading == true)
        #expect(model.loadTrigger.amount == "250")
        #expect(model.loadTrigger.isImmediate == true)

        model.onChangeAmountText("", text: "250")

        #expect(model.loadTrigger.amount == "250")
        #expect(model.loadTrigger.isImmediate == true)
    }

    @Test
    func sellSceneUsesSellDefaultLoadTriggerAmount() {
        let model = FiatSceneViewModelTests.mock(type: .sell)

        #expect(model.loadTrigger.type == .sell)
        #expect(model.loadTrigger.amount == "100")
    }

    @Test
    func fiatProviderRowsUseUsdPriceSource() {
        let model = FiatSceneViewModelTests.mock()
        FiatSceneViewModelTests.load(model, quotes: [.mock(fiatAmount: 50, cryptoAmount: 0.000488)])
        model.priceUsdQuery.value = 100_000

        let row = model.fiatProviderViewModel.state.value?.items.first

        #expect(row?.assetPrice == 100_000)
        #expect(row?.subtitleExtra == "$48.80")
    }
}
