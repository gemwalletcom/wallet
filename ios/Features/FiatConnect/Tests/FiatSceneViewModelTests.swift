// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import struct Gemstone.FiatQuote
import struct Gemstone.GemFiatQuoteRequest
import GemstoneServicesTestKit
import BigInt
@testable import FiatConnect
import FiatConnectTestKit
import Formatters
import Foundation
import Localization
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing

@MainActor
final class FiatSceneViewModelTests {
    @Test
    func defaultAmount() {
        let model = FiatSceneViewModel.mock()
        #expect(model.amount == "50")

        model.type = .sell

        #expect(model.amount == "100")
    }

    @Test
    func selectBuyAmount() {
        let model = FiatSceneViewModel.mock()
        model.onSelect(amount: 150)

        #expect(model.amount == "150")

        model.onSelect(amount: 1)

        #expect(model.amount == "1")
    }

    @Test
    func selectSellAmount() {
        let model = FiatSceneViewModel.mock()
        model.type = .sell

        model.onSelect(amount: 50)

        #expect(model.amount == "50")

        model.onSelect(amount: 100)

        #expect(model.amount == "100")
    }

    @Test
    func testCurrencySymbol() {
        let model = FiatSceneViewModel.mock()
        #expect(model.currencyInputConfig.currencySymbol == "$")

        model.type = .sell

        #expect(model.currencyInputConfig.currencySymbol == "$")
    }

    @Test
    func buttonsTitle() {
        let model = FiatSceneViewModel.mock()

        #expect(model.buttonTitle(amount: 10) == "$10")

        model.type = .sell

        #expect(model.buttonTitle(amount: 100) == "$100")
    }

    @Test
    func assetBalanceIncludesSymbol() {
        let asset = Asset.mockTron()
        let model = FiatSceneViewModel.mock(assetAddress: .mock(asset: asset))

        model.assetQuery.value = .mock(
            asset: asset,
            balance: .mock(available: BigInt(66_670_000)),
        )

        #expect(model.assetBalance == "66.67 TRX")
    }

    @Test
    func showFiatTypePickerWhenSellEnabledWithZeroBalance() {
        let model = FiatSceneViewModel.mock()

        model.assetQuery.value = .mock(
            balance: .zero,
            metadata: .mock(isSellEnabled: true),
        )

        #expect(model.showFiatTypePicker)
    }

    @Test
    func unsupportedSellRouteFallsBackToBuyWithAmount() {
        let model = FiatSceneViewModel.mock(type: .sell, amount: 40)
        let previousAssetData = model.assetData
        let unsupportedAssetData = AssetData.mock(metadata: .mock(isSellEnabled: false))
        model.assetQuery.value = unsupportedAssetData
        model.onAssetDataChange(previousAssetData, unsupportedAssetData)

        #expect(!model.showFiatTypePicker)
        #expect(model.type == .buy)
        #expect(model.session.sell.amount == "40")
        #expect(model.session.buy.amount == "40")
        #expect(model.amount == "40")
        #expect(model.loadTrigger?.request == GemFiatQuoteRequest(quoteType: .buy, amount: 40))
        #expect(model.loadTrigger?.isImmediate == true)
        #expect(model.title == Localized.Buy.title(model.asset.name))
    }

    @Test
    func testRateValue() {
        let model = FiatSceneViewModel.mock()
        model.session = model.session.onQuoteResults(results: .mock(quotes: [.mock(fiatAmount: 1200, cryptoAmount: 2.0)]))

        #expect(model.rateValue == "1 \(model.asset.symbol) ≈ $600.00")
        #expect(model.cryptoAmountValue == "≈ 2 BTC")
    }

    @Test
    func balanceChangeReachesTheSession() {
        let asset = Asset.mockEthereumUSDT()
        let model = FiatSceneViewModel.mock(assetAddress: .mock(asset: asset), type: .sell)

        model.onAssetDataChange(
            .mock(asset: asset),
            .mock(asset: asset, balance: .mock(available: BigInt(415_650_000))),
        )

        #expect(model.session.available == 415_650_000)
    }

    @Test
    func selectingProviderRevalidatesSellBalance() {
        let affordable = FiatQuote.mock(fiatAmount: 100, cryptoAmount: 1, type: .sell)
        let unaffordable = FiatQuote.mock(fiatAmount: 100, cryptoAmount: 3, type: .sell, providerId: .transak)
        let model = FiatSceneViewModel.mock(type: .sell)

        model.onAssetDataChange(
            .mock(),
            .mock(balance: .mock(available: BigInt(200_000_000))),
        )
        model.session = model.session.onQuoteResults(results: .mock(quotes: [affordable, unaffordable], amount: 100, type: .sell))

        #expect(model.selectedQuote(model.viewState)?.quoteId == affordable.id)
        #expect(model.allowSelectProvider(model.viewState))
        #expect(model.amountError == nil)
        #expect(model.actionButtonState(model.viewState) == .normal)

        model.onSelectQuotes([FiatQuoteViewModel(asset: model.asset, row: .mock(provider: .transak))])

        #expect(model.selectedQuote(model.viewState)?.quoteId == unaffordable.id)
        #expect(model.amountError?.localizedDescription == Localized.Transfer.insufficientBalance("**\(model.asset.name) (\(model.asset.symbol))**"))
        #expect(model.actionButtonState(model.viewState) == .disabled)
        #expect(!model.isPresentingFiatProvider)
    }

    @Test
    func actionButtonStateFollowsTheSession() {
        let model = FiatSceneViewModel.mock()
        #expect(model.actionButtonState(model.viewState) == .loading(showProgress: true))

        model.session = model.session.onQuoteResults(results: .mock())
        #expect(model.actionButtonState(model.viewState) == .disabled)
        #expect(model.emptyTitle(model.viewState) == Localized.Buy.noResults)

        model.amount = "0"
        #expect(model.actionButtonState(model.viewState) == .disabled)
        #expect(model.emptyTitle(model.viewState) == Localized.Input.enterAmountTo(Localized.Wallet.buy))

        model.amount = "100"
        model.session = model.session.onQuoteResults(results: .mock(quotes: [.mock(fiatAmount: 100, cryptoAmount: 1)], amount: 100))
        #expect(model.actionButtonState(model.viewState) == .normal)
        #expect(model.actionButtonTitle(model.viewState) == Localized.Common.continue)

        model.urlState = .loading
        #expect(model.actionButtonState(model.viewState) == .loading(showProgress: true))
    }

    @Test
    func failedQuotesOfferARetry() {
        let model = FiatSceneViewModel.mock()
        model.session = model.session.onQuoteResults(results: .mock(error: .Api(msg: "offline")))

        #expect(model.quotesState(model.viewState).isError)
        #expect(model.actionButtonTitle(model.viewState) == Localized.Common.tryAgain)
        #expect(model.actionButtonState(model.viewState) == .normal)
    }

    @Test
    func urlStateInitialValue() {
        let model = FiatSceneViewModel.mock()

        #expect(model.urlState.isNoData == true)
        #expect(model.urlState.isLoading == false)
    }

    @Test
    func loadTriggerOnChangeTypeIsImmediate() {
        let model = FiatSceneViewModel.mock()

        model.type = .sell
        model.onChangeType(oldType: .buy, newType: .sell)

        #expect(model.loadTrigger?.request == GemFiatQuoteRequest(quoteType: .sell, amount: 100))
        #expect(model.loadTrigger?.isImmediate == true)
    }

    @Test
    func loadTriggerOnSelectAmountIsImmediate() {
        let model = FiatSceneViewModel.mock()

        model.onSelect(amount: 250)

        #expect(model.loadTrigger?.request == GemFiatQuoteRequest(quoteType: .buy, amount: 250))
        #expect(model.loadTrigger?.isImmediate == true)
    }

    @Test
    func typingTheAmountIsDebounced() {
        let model = FiatSceneViewModel.mock()

        model.amount = "123"

        #expect(model.loadTrigger?.request == GemFiatQuoteRequest(quoteType: .buy, amount: 123))
        #expect(model.loadTrigger?.isImmediate == false)
        #expect(model.amount == "123")
    }

    @Test
    func loadTriggerOnSelectRandomAmountIsImmediate() {
        let model = FiatSceneViewModel.mock()

        model.onSelectRandomAmount()

        #expect(model.loadTrigger?.isImmediate == true)
    }

    @Test
    func amountOutsideTheConfiguredRangeSchedulesNoFetch() {
        let model = FiatSceneViewModel.mock()

        model.amount = "4"

        #expect(model.loadTrigger == nil)
        #expect(model.amountError?.localizedDescription == Localized.Transfer.minimumAmount("$5.00"))
    }

    @Test
    func presetSelectionDoesNotScheduleSecondDebouncedFetch() {
        let model = FiatSceneViewModel.mock()
        model.session = model.session.onQuoteResults(results: .mock(error: .Api(msg: "offline")))

        model.onSelect(amount: 250)

        #expect(model.amount == "250")
        #expect(model.quotesState(model.viewState).isLoading == true)
        #expect(model.loadTrigger?.request == GemFiatQuoteRequest(quoteType: .buy, amount: 250))
        #expect(model.loadTrigger?.isImmediate == true)

        model.amount = "250"

        #expect(model.loadTrigger?.request == GemFiatQuoteRequest(quoteType: .buy, amount: 250))
        #expect(model.loadTrigger?.isImmediate == true)
    }

    @Test
    func sellSceneUsesSellDefaultLoadTriggerAmount() {
        let model = FiatSceneViewModel.mock(type: .sell)

        #expect(model.loadTrigger?.request == GemFiatQuoteRequest(quoteType: .sell, amount: 100))
    }

    @Test
    func everyPhaseMapsToItsMessage() {
        let model = FiatSceneViewModel.mock()

        model.amount = .empty
        #expect(model.amountError == nil)

        model.amount = "0"
        #expect(model.amountError == nil)

        model.amount = "."
        #expect(model.amountError?.localizedDescription == Localized.Errors.invalidAmount)

        model.amount = "4"
        #expect(model.amountError?.localizedDescription == Localized.Transfer.minimumAmount("$5.00"))

        model.amount = "10001"
        #expect(model.amountError?.localizedDescription == Localized.Transfer.maximumAmount("$10,000.00"))

        model.amount = "100"
        #expect(model.quotesState(model.viewState).isLoading)
        #expect(model.amountError == nil)

        model.session = model.session.onQuoteResults(results: .mock(quotes: [.mock(fiatAmount: 100, cryptoAmount: 1)], amount: 100))
        #expect(model.selectedQuote(model.viewState) != nil)
        #expect(model.amountError == nil)

        model.amount = "200"
        model.session = model.session.onQuoteResults(results: .mock(amount: 200))
        #expect(model.quotesState(model.viewState).isNoData)
        #expect(model.amountError == nil)

        model.amount = "300"
        model.session = model.session.onQuoteResults(results: .mock(amount: 300, error: .Api(msg: "offline")))
        #expect(model.quotesState(model.viewState).isError)
        #expect(model.amountError == nil)
    }

    @Test
    func onlyAFetchableAmountCarriesATrigger() {
        let model = FiatSceneViewModel.mock()

        for unfetchable in [String.empty, "0", ".", "4", "10001"] {
            model.amount = unfetchable
            #expect(model.loadTrigger == nil, "\(unfetchable) should not schedule a fetch")
        }

        for fetchable in ["5", "100", "10000"] {
            model.amount = fetchable
            #expect(model.loadTrigger?.request.amount == Double(fetchable))
        }
    }

    @Test
    func aBalanceOrProviderChangeDoesNotRefetch() {
        let model = FiatSceneViewModel.mock(type: .sell)
        model.session = model.session.onQuoteResults(results: .mock(quotes: [.mock(cryptoAmount: 1, type: .sell), .mock(cryptoAmount: 2, type: .sell, providerId: .transak)], amount: 100, type: .sell))
        let trigger = model.loadTrigger

        model.onAssetDataChange(.mock(), .mock(balance: .mock(available: BigInt(500_000_000))))
        #expect(model.loadTrigger == trigger)

        model.onSelectQuotes([FiatQuoteViewModel(asset: model.asset, row: .mock(provider: .transak))])
        #expect(model.loadTrigger == trigger)
    }

    @Test
    func eachTypeKeepsItsOwnAmount() {
        let model = FiatSceneViewModel.mock()

        model.amount = "75"
        model.type = .sell
        #expect(model.amount == "100")

        model.amount = "125"
        model.type = .buy
        #expect(model.amount == "75")
    }

    @Test
    func onlyAWholeAmountIsAccepted() {
        let model = FiatSceneViewModel.mock()

        #expect(model.currencyInputConfig.keyboardType == .numberPad)

        model.amount = "12.5"
        #expect(model.amountError?.localizedDescription == Localized.Errors.invalidAmount)
        #expect(model.loadTrigger == nil)

        model.amount = "12"
        #expect(model.amountError == nil)
        #expect(model.loadTrigger?.request == GemFiatQuoteRequest(quoteType: .buy, amount: 12))
    }

    @Test
    func fiatProviderRowsUseUsdPriceSource() {
        let model = FiatSceneViewModel.mock()
        model.session = model.session.onQuoteResults(results: .mock(quotes: [.mock(fiatAmount: 50, cryptoAmount: 0.000488)]))
        model.priceUsdQuery.value = 100_000

        let row = model.fiatProviderViewModel.state.value?.items.first

        #expect(row?.subtitleExtra == "$48.80")
    }
}
