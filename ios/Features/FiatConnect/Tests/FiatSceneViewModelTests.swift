// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import protocol Gemstone.GemFiatServiceProtocol
import GemstoneServicesTestKit
import BigInt
@testable import FiatConnect
import GemstoneServices
import Formatters
import Foundation
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing

@MainActor
final class FiatSceneViewModelTests {
    private static func mock(
        fiatService: any GemFiatServiceProtocol = GemFiatServiceMock(),
        currencyFormatter: CurrencyFormatter = .init(locale: Locale.US, currencyCode: Currency.usd.rawValue),
        assetAddress: AssetAddress = .mock(),
        wallet: Wallet = .mock(),
        type: FiatQuoteType = .buy,
        amount: Int? = nil,
    ) -> FiatSceneViewModel {
        FiatSceneViewModel(
            fiatService: fiatService,
            currencyFormatter: currencyFormatter,
            assetAddress: assetAddress,
            wallet: wallet,
            balanceService: .mock(),
            type: type,
            amount: amount,
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
    func testRateValue() {
        let model = FiatSceneViewModelTests.mock()
        let quote = FiatQuote.mock(fiatAmount: 1200, cryptoAmount: 2.0, type: model.type)
        model.buyViewModel.selectedQuote = quote

        #expect(model.rateValue == "1 \(model.asset.symbol) ≈ $600.00")
    }

    @Test
    func fiatValidation() {
        let model = FiatSceneViewModelTests.mock()

        model.inputValidationModel.text = "4"
        #expect(model.inputValidationModel.update() == false)

        model.inputValidationModel.text = "5"
        #expect(model.inputValidationModel.update() == true)

        model.inputValidationModel.text = "10000"
        #expect(model.inputValidationModel.update() == true)

        model.inputValidationModel.text = "10001"
        #expect(model.inputValidationModel.update() == false)
    }

    @Test
    func sellFiatValidation() {
        let model = FiatSceneViewModelTests.mock()
        model.type = .sell

        model.inputValidationModel.text = "4"
        #expect(model.inputValidationModel.update() == false)

        model.inputValidationModel.text = "5"
        #expect(model.inputValidationModel.update() == true)

        model.inputValidationModel.text = "10000"
        #expect(model.inputValidationModel.update() == true)

        model.inputValidationModel.text = "10001"
        #expect(model.inputValidationModel.update() == false)
    }

    @Test
    func sellFiatValidationRefreshesAfterBalanceChange() {
        let asset = Asset.mockEthereumUSDT()
        let model = FiatSceneViewModelTests.mock(
            assetAddress: .mock(asset: asset),
            type: .sell,
        )
        let quote = FiatQuote.mock(fiatAmount: 100, cryptoAmount: 104.97, type: .sell)
        model.sellViewModel.selectedQuote = quote
        model.inputValidationModel.text = "100"

        #expect(model.inputValidationModel.update() == false)

        model.onAssetDataChange(
            .mock(asset: asset),
            .mock(asset: asset, balance: .mock(available: BigInt(415_650_000))),
        )

        #expect(model.buyViewModel.availableBalance == BigInt(415_650_000))
        #expect(model.sellViewModel.availableBalance == BigInt(415_650_000))
        #expect(model.inputValidationModel.update() == true)
    }

    @Test
    func selectingProviderRevalidatesSellBalance() {
        let asset = Asset.mockEthereumUSDT()
        let formatter = CurrencyFormatter(locale: .US, currencyCode: Currency.usd.rawValue)
        let affordable = FiatQuote.mock(fiatAmount: 100, cryptoAmount: 100, type: .sell)
        let unaffordable = FiatQuote.mock(fiatAmount: 100, cryptoAmount: 300, type: .sell)
        let model = FiatSceneViewModelTests.mock(
            assetAddress: .mock(asset: asset),
            type: .sell,
        )

        model.onAssetDataChange(
            .mock(asset: asset),
            .mock(asset: asset, balance: .mock(available: BigInt(200_000_000))),
        )
        model.inputValidationModel.text = "100"
        model.sellViewModel.quotesState = .data(FiatQuotes(amount: 100, quotes: [affordable, unaffordable]))
        model.sellViewModel.selectedQuote = affordable

        #expect(model.actionButtonState.value != nil)

        model.onSelectQuotes([FiatQuoteViewModel(asset: asset, quote: unaffordable, formatter: formatter)])

        #expect(model.inputValidationModel.isInvalid)
        #expect(model.actionButtonState.value == nil)
    }

    @Test
    func actionButtonStateInvalidInput() {
        let model = FiatSceneViewModelTests.mock()
        model.buyViewModel.quotesState = .data(FiatQuotes(amount: 100, quotes: []))

        model.inputValidationModel.text = "4"
        model.inputValidationModel.update()

        #expect(model.actionButtonState.value == nil)
    }

    @Test
    func actionButtonStateLoading() {
        let model = FiatSceneViewModelTests.mock()
        model.buyViewModel.quotesState = .loading

        model.inputValidationModel.text = "100"
        model.inputValidationModel.update()

        #expect(model.actionButtonState.value == nil)
    }

    @Test
    func actionButtonStateValidWithQuote() {
        let model = FiatSceneViewModelTests.mock()
        let quote = FiatQuote.mock(fiatAmount: 100, cryptoAmount: 1, type: .buy)

        model.buyViewModel.quotesState = .data(FiatQuotes(amount: 100, quotes: [quote]))
        model.buyViewModel.selectedQuote = quote
        model.inputValidationModel.text = "100"
        model.inputValidationModel.update()

        #expect(model.actionButtonState.value != nil)
    }

    @Test
    func actionButtonStateValidNoQuote() {
        let model = FiatSceneViewModelTests.mock()
        model.buyViewModel.quotesState = .noData

        model.inputValidationModel.text = "100"
        model.inputValidationModel.update()

        #expect(model.actionButtonState.value == nil)
    }

    @Test
    func urlStateBlocksButton() {
        let model = FiatSceneViewModelTests.mock()
        let quote = FiatQuote.mock(fiatAmount: 100, cryptoAmount: 1, type: .buy)

        model.buyViewModel.quotesState = .data(FiatQuotes(amount: 100, quotes: [quote]))
        model.buyViewModel.selectedQuote = quote
        model.inputValidationModel.text = "100"
        model.inputValidationModel.update()

        #expect(model.actionButtonState.value != nil)

        model.urlState = .loading

        #expect(model.actionButtonState.value == nil)
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
        model.buyViewModel.quotesState = .error(NSError(domain: "test", code: 1))

        model.onSelect(amount: 250)

        #expect(model.buyViewModel.amount == "250")
        #expect(model.buyViewModel.inputValidationModel.text == "250")
        #expect(model.buyViewModel.quotesState.isLoading == true)
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

    // MARK: - ShouldSkipFetch Tests

    @Test
    func secondFetchSkippedWhenSameAmountLoading() {
        let model = FiatSceneViewModelTests.mock()

        model.buyViewModel.loadingAmount = 100.0

        #expect(model.buyViewModel.shouldSkipFetch(for: 100.0) == true)
    }

    @Test
    func fetchAllowedForDifferentAmount() {
        let model = FiatSceneViewModelTests.mock()

        model.buyViewModel.loadingAmount = 100.0
        model.buyViewModel.quotesState = .data(FiatQuotes(amount: 100.0, quotes: []))

        #expect(model.buyViewModel.shouldSkipFetch(for: 200.0) == false)
    }

    @Test
    func fiatProviderRowsUseUsdPriceSource() {
        let model = FiatSceneViewModelTests.mock()
        let quote = FiatQuote.mock(fiatAmount: 50, cryptoAmount: 0.000488, type: .buy)
        model.buyViewModel.quotesState = .data(FiatQuotes(amount: 50, quotes: [quote]))
        model.priceUsdQuery.value = 100_000

        let row = model.fiatProviderViewModel.state.value?.items.first

        #expect(row?.assetPrice == 100_000)
        #expect(row?.subtitleExtra == "$48.80")
    }
}
