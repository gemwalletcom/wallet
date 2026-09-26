// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
@testable import FiatConnect
import FiatConnectTestKit
import Formatters
import Foundation
import struct Gemstone.FiatQuote
import func Gemstone.formattedAmount
import func Gemstone.formattedCurrency
import struct Gemstone.GemFiatQuoteRequest
import struct Gemstone.GemProviderRow
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
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
        #expect(model.currencyInputConfig(model.viewState).currencySymbol == "$")

        model.type = .sell

        #expect(model.currencyInputConfig(model.viewState).currencySymbol == "$")
    }

    @Test
    func suggestedAmountsCarryTheCurrencySymbol() {
        let model = FiatSceneViewModel.mock()

        #expect(model.suggestedAmounts.map { $0.value.text() } == ["$100", "$250"])

        model.type = .sell

        #expect(model.suggestedAmounts.map(\.amount) == [100, 250])
    }

    @Test
    func assetBalanceIncludesSymbol() {
        let asset = Asset.mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6)
        let model = FiatSceneViewModel.mock(assetAddress: .mock(asset: asset))

        model.assetQuery.value = .mock(
            asset: asset,
            balance: .mock(available: BigInt(66_670_000)),
        )

        #expect(model.assetBalance == "66.67 TRX")
    }

    @Test
    func aZeroBalanceShows() {
        let asset = Asset.mock(id: .mock(chain: .tron), name: "TRON", symbol: "TRX", decimals: 6)
        let model = FiatSceneViewModel.mock(assetAddress: .mock(asset: asset))

        model.assetQuery.value = .mock(asset: asset, balance: .zero)

        #expect(model.assetBalance == "0 TRX")
    }

    @Test
    func theTypePickerShowsWhenSellIsEnabledWithZeroBalance() {
        let model = FiatSceneViewModel.mock()

        model.assetQuery.value = .mock(
            balance: .zero,
            metadata: .mock(isSellEnabled: true),
        )

        #expect(model.viewState.showsTypePicker)
    }

    @Test
    func unsupportedSellRouteFallsBackToBuyWithAmount() {
        let model = FiatSceneViewModel.mock(type: .sell, amount: 40)
        let previousAssetData = model.assetData
        let unsupportedAssetData = AssetData.mock(metadata: .mock(isSellEnabled: false))
        model.assetQuery.value = unsupportedAssetData
        model.onAssetDataChange(previousAssetData, unsupportedAssetData)

        #expect(!model.viewState.showsTypePicker)
        #expect(model.type == .buy)
        #expect(model.amount == "40")
        #expect(model.loadTrigger?.request == GemFiatQuoteRequest(quoteType: .buy, amount: 40))
        #expect(model.loadTrigger?.isImmediate == true)
        #expect(model.title == Localized.Buy.title(model.asset.name))
    }

    @Test
    func rateValue() {
        let asset = Asset.mock(name: "Bitcoin", symbol: "BTC", decimals: 8)
        let model = FiatSceneViewModel.mock(assetAddress: .mock(asset: asset))
        model.session = model.session.onQuoteResults(results: .mock(
            request: .mock(quoteType: .buy, amount: 50),
            quotes: [.mock(asset: asset.toGem(), provider: .mock(id: .moonPay, enabled: true, buyEnabled: true, sellEnabled: true), fiatAmount: 1200, fiatCurrency: "USD", cryptoAmount: 2.0)],
        ))

        guard case let .rate(_, rate, _) = model.viewState.rateRow else {
            Issue.record("a selected quote shows its rate")
            return
        }
        #expect(rate.text(formattedValue: rate.value.text()) == "1 \(model.asset.symbol) ≈ $600.00")
        #expect(model.cryptoAmountValue(model.viewState) == "≈ 2 BTC")
    }

    @Test
    func balanceChangeReachesTheSession() {
        let asset = Asset.mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)
        let model = FiatSceneViewModel.mock(assetAddress: .mock(asset: asset), type: .sell)

        model.onAssetDataChange(
            .mock(asset: asset),
            .mock(asset: asset, balance: .mock(available: BigInt(415_650_000))),
        )

        #expect(model.session.available == 415_650_000)
    }

    @Test
    func selectingProviderRevalidatesSellBalance() {
        let asset = Asset.mock(name: "Bitcoin", symbol: "BTC", decimals: 8)
        let affordable = FiatQuote.mock(asset: asset.toGem(), provider: .mock(id: .moonPay, enabled: true, buyEnabled: true, sellEnabled: true), quoteType: .sell, fiatAmount: 100, fiatCurrency: "USD", cryptoAmount: 1)
        let unaffordable = FiatQuote.mock(asset: asset.toGem(), provider: .mock(id: .transak, enabled: true, buyEnabled: true, sellEnabled: true), quoteType: .sell, fiatAmount: 100, fiatCurrency: "USD", cryptoAmount: 3)
        let model = FiatSceneViewModel.mock(assetAddress: .mock(asset: asset), type: .sell)

        model.onAssetDataChange(
            .mock(asset: asset, metadata: .mock(isSellEnabled: true)),
            .mock(asset: asset, balance: .mock(available: BigInt(200_000_000)), metadata: .mock(isSellEnabled: true)),
        )
        model.session = model.session.onQuoteResults(results: .mock(request: .mock(quoteType: .sell, amount: 100), quotes: [affordable, unaffordable]))

        #expect(model.viewState.selectedQuoteRow?.quoteId == affordable.id)
        #expect(model.viewState.canSelectProvider)
        #expect(model.amountError(model.viewState) == nil)
        #expect(model.viewState.buttonState.state == .normal)

        model.onSelectQuotes([GemProviderRow(kind: .fiat(provider: .transak), name: "Transak", amount: formattedAmount(value: 0, symbol: "BTC", style: .auto), fiat: nil, isSelected: false)])

        #expect(model.viewState.selectedQuoteRow?.quoteId == unaffordable.id)
        #expect(model.amountError(model.viewState)?.localizedDescription == Localized.Transfer.insufficientBalance("**\(model.asset.name) (\(model.asset.symbol))**"))
        #expect(model.viewState.buttonState.state == .disabled)
        #expect(!model.isPresentingFiatProvider)
    }

    @Test
    func actionButtonStateFollowsTheSession() {
        let model = FiatSceneViewModel.mock()
        #expect(model.viewState.buttonState.state == .loading(showProgress: true))

        model.session = model.session.onQuoteResults(results: .mock(request: .mock(quoteType: .buy, amount: 50)))
        #expect(model.viewState.buttonState.state == .disabled)
        #expect(model.emptyTitle(model.viewState) == Localized.Buy.noResults)

        model.amount = "0"
        #expect(model.viewState.buttonState.state == .disabled)
        #expect(model.emptyTitle(model.viewState) == Localized.Input.enterAmountTo(Localized.Wallet.buy))

        model.amount = "100"
        model.session = model.session.onQuoteResults(results: .mock(
            request: .mock(quoteType: .buy, amount: 100),
            quotes: [.mock(provider: .mock(id: .moonPay, enabled: true, buyEnabled: true, sellEnabled: true), fiatAmount: 100, fiatCurrency: "USD", cryptoAmount: 1)],
        ))
        #expect(model.viewState.buttonState.state == .normal)
        #expect(model.viewState.buttonAction.title == Localized.Common.continue)

        model.urlState = .loading
        #expect(model.viewState.buttonState.state == .loading(showProgress: true))
    }

    @Test
    func aFailedQuoteReadsAsAnErrorAndItsActionAsRetry() {
        let model = FiatSceneViewModel.mock()
        model.session = model.session.onQuoteResults(results: .mock(request: .mock(quoteType: .buy, amount: 50), error: .Api(msg: "offline")))

        #expect(model.quotesState(model.viewState).isError)
        #expect(model.viewState.buttonAction.title == Localized.Common.tryAgain)
    }

    @Test
    func aFailedQuoteStopsTheClockUntilTheAmountChanges() async {
        let model = FiatSceneViewModel.mock()
        model.session = model.session.onQuoteResults(results: .mock(request: .mock(quoteType: .buy, amount: 50), error: .Api(msg: "offline")))

        await model.refreshQuotes()
        #expect(model.viewState.buttonAction == .retryQuote, "the timer leaves the failure to the retry button")

        model.onSelect(amount: 150)
        await model.refreshQuotes()
        #expect(model.viewState.buttonAction == .continue, "a new amount starts the clock again")
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
        #expect(model.amountError(model.viewState)?.localizedDescription == Localized.Transfer.minimumAmount("$5.00"))
    }

    @Test
    func presetSelectionDoesNotScheduleSecondDebouncedFetch() {
        let model = FiatSceneViewModel.mock()
        model.session = model.session.onQuoteResults(results: .mock(request: .mock(quoteType: .buy, amount: 50), error: .Api(msg: "offline")))

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
        #expect(model.amountError(model.viewState) == nil)

        model.amount = "0"
        #expect(model.amountError(model.viewState) == nil)

        model.amount = "."
        #expect(model.amountError(model.viewState)?.localizedDescription == Localized.Errors.invalidAmount)

        model.amount = "4"
        #expect(model.amountError(model.viewState)?.localizedDescription == Localized.Transfer.minimumAmount("$5.00"))

        model.amount = "10001"
        #expect(model.amountError(model.viewState)?.localizedDescription == Localized.Transfer.maximumAmount("$10,000.00"))

        model.amount = "100"
        #expect(model.quotesState(model.viewState).isLoading)
        #expect(model.amountError(model.viewState) == nil)

        model.session = model.session.onQuoteResults(results: .mock(
            request: .mock(quoteType: .buy, amount: 100),
            quotes: [.mock(provider: .mock(id: .moonPay, enabled: true, buyEnabled: true, sellEnabled: true), fiatAmount: 100, fiatCurrency: "USD", cryptoAmount: 1)],
        ))
        #expect(model.viewState.selectedQuoteRow != nil)
        #expect(model.amountError(model.viewState) == nil)

        model.amount = "200"
        model.session = model.session.onQuoteResults(results: .mock(request: .mock(quoteType: .buy, amount: 200)))
        #expect(model.quotesState(model.viewState).isNoData)
        #expect(model.amountError(model.viewState) == nil)

        model.amount = "300"
        model.session = model.session.onQuoteResults(results: .mock(request: .mock(quoteType: .buy, amount: 300), error: .Api(msg: "offline")))
        #expect(model.quotesState(model.viewState).isError)
        #expect(model.amountError(model.viewState) == nil)
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
        model.session = model.session.onQuoteResults(results: .mock(
            request: .mock(quoteType: .sell, amount: 100),
            quotes: [
                .mock(provider: .mock(id: .moonPay, enabled: true, buyEnabled: true, sellEnabled: true), quoteType: .sell, fiatCurrency: "USD", cryptoAmount: 1),
                .mock(provider: .mock(id: .transak, enabled: true, buyEnabled: true, sellEnabled: true), quoteType: .sell, fiatCurrency: "USD", cryptoAmount: 2),
            ],
        ))
        let trigger = model.loadTrigger

        model.onAssetDataChange(.mock(metadata: .mock(isSellEnabled: true)), .mock(balance: .mock(available: BigInt(500_000_000)), metadata: .mock(isSellEnabled: true)))
        #expect(model.loadTrigger == trigger)

        model.onSelectQuotes([GemProviderRow(kind: .fiat(provider: .transak), name: "Transak", amount: formattedAmount(value: 0, symbol: "BTC", style: .auto), fiat: nil, isSelected: false)])
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

        #expect(model.currencyInputConfig(model.viewState).keyboardType == .numberPad)

        model.amount = "12.5"
        #expect(model.amountError(model.viewState)?.localizedDescription == Localized.Errors.invalidAmount)
        #expect(model.loadTrigger == nil)

        model.amount = "12"
        #expect(model.amountError(model.viewState) == nil)
        #expect(model.loadTrigger?.request == GemFiatQuoteRequest(quoteType: .buy, amount: 12))
    }

    @Test
    func fiatProviderRowsUseUsdPriceSource() {
        let model = FiatSceneViewModel.mock()
        model.session = model.session.onQuoteResults(results: .mock(
            request: .mock(quoteType: .buy, amount: 50),
            quotes: [.mock(provider: .mock(id: .moonPay, enabled: true, buyEnabled: true, sellEnabled: true), fiatAmount: 50, fiatCurrency: "USD", cryptoAmount: 0.000488)],
        ))
        model.priceUsdQuery.value = 100_000

        let row = model.fiatProviderViewModel.state.value?.items.first

        #expect(row?.listItem.subtitleExtra == "$48.80")
    }
}
