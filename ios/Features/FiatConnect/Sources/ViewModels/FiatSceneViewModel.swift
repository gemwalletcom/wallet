// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import enum Gemstone.FiatProviderName
import enum Gemstone.GemFiatAmountCheck
import struct Gemstone.GemFiatQuoteRow
import protocol Gemstone.GemFiatQuoteServiceProtocol
import struct Gemstone.GemFiatQuotesResult
import struct Gemstone.GemFiatSession
import struct Gemstone.GemFiatSuggestedAmount
import struct Gemstone.GemFiatViewState
import enum Gemstone.GemServiceError
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@MainActor
@Observable
public final class FiatSceneViewModel {
    private let service: any GemFiatQuoteServiceProtocol

    var quoteDebounce: Duration {
        .milliseconds(service.quoteDebounceMilliseconds())
    }

    var quoteRefreshInterval: TimeInterval {
        TimeInterval(service.quoteRefreshIntervalMilliseconds()) / 1000
    }

    private let wallet: Wallet
    private let assetAddress: AssetAddress
    private let currencyFormatter: CurrencyFormatter
    private let locale: Locale
    private let valueFormatter = ValueFormatter(locale: .US, style: .auto)

    public let priceUsdQuery: ObservableQuery<PriceUsdRequest>
    public let assetQuery: ObservableQuery<AssetRequest>
    var assetData: AssetData {
        assetQuery.value
    }

    var session: GemFiatSession
    var urlState: StateViewType<Void> = .noData
    var isPresentingFiatProvider: Bool = false
    var isPresentingAlertMessage: AlertMessage?
    var loadTrigger: FiatLoadTrigger?

    public init(
        service: any GemFiatQuoteServiceProtocol,
        assetAddress: AssetAddress,
        wallet: Wallet,
        type: FiatQuoteType = .buy,
        amount: Int? = nil,
        locale: Locale = .current,
    ) {
        self.service = service
        self.locale = locale
        currencyFormatter = CurrencyFormatter(locale: locale, currencyCode: service.currency.rawValue)
        self.assetAddress = assetAddress
        self.wallet = wallet
        assetQuery = ObservableQuery(AssetRequest(walletId: wallet.id, assetId: assetAddress.asset.id), initialValue: .with(asset: assetAddress.asset))
        priceUsdQuery = ObservableQuery(PriceUsdRequest(assetId: assetAddress.asset.id), initialValue: nil)
        session = service.newSession(type: type, amount: amount)
        loadTrigger = FiatLoadTrigger(session: session, isImmediate: true)
    }

    var type: FiatQuoteType {
        get { session.type }
        set { session = session.onTypeChanged(quoteType: newValue.toGem()) }
    }

    var viewState: GemFiatViewState {
        session.viewState(assetPrice: priceUsdQuery.value, isUrlLoading: urlState.isLoading, isSellEnabled: assetData.metadata.isSellEnabled)
    }

    var amount: String {
        get { viewState.amount }
        set { applyAmount(newValue, isImmediate: false) }
    }

    var amountError: (any Error)? {
        switch viewState.phase {
        case .noInput, .loading, .noQuotes, .failed: nil
        case .invalidInput: viewState.phase.inputErrorText.map(AnyError.init)
        case let .invalid(check): check.errorText(locale: locale).map(AnyError.init)
        case .ready: viewState.amountCheck.errorText(locale: locale).map(AnyError.init)
        }
    }

    func providerModel(_ viewState: GemFiatViewState) -> FiatProviderViewModel {
        FiatProviderViewModel(
            quotesState: quotesState(viewState),
            emptyTitle: emptyTitle(viewState),
            selectedQuote: selectedQuote(viewState),
            allowSelectProvider: allowSelectProvider(viewState),
            rateRow: viewState.rateRow,
        )
    }

    func quotesState(_ viewState: GemFiatViewState) -> StateViewType<[GemFiatQuoteRow]> {
        switch viewState.phase {
        case .noInput, .invalidInput, .invalid, .noQuotes: .noData
        case .loading: .loading
        case .ready: .data(viewState.quoteRows)
        case let .failed(error): .error(error)
        }
    }

    func selectedQuote(_ viewState: GemFiatViewState) -> GemFiatQuoteRow? {
        viewState.selectedQuoteRow
    }

    var title: String {
        switch type {
        case .buy, .sell: type.title(asset: asset.name)
        }
    }

    func allowSelectProvider(_ viewState: GemFiatViewState) -> Bool {
        viewState.canSelectProvider
    }

    var currencyInputConfig: any CurrencyInputConfigurable {
        FiatCurrencyInputConfig(
            secondaryText: cryptoAmountValue,
            currencySymbol: currencyFormatter.symbol,
            numberFormat: NumberInput.format(locale),
        )
    }

    func actionButtonTitle(_ viewState: GemFiatViewState) -> String {
        viewState.buttonAction.title
    }

    func actionButtonState(_ viewState: GemFiatViewState) -> ButtonState {
        viewState.buttonState.state
    }

    var providerTitle: String {
        Localized.Common.provider
    }

    var errorTitle: String {
        Localized.Errors.errorOccurred
    }

    func emptyTitle(_ viewState: GemFiatViewState) -> String {
        viewState.quotesMessage()?.title(action: type.action) ?? .empty
    }

    var assetTitle: String {
        asset.name
    }

    var typeAmountButtonTitle: String {
        Emoji.random
    }

    var asset: Asset {
        assetAddress.asset
    }

    var assetImage: AssetImage {
        AssetIdViewModel(assetId: asset.id).assetImage
    }

    var suggestedAmounts: [GemFiatSuggestedAmount] {
        service.suggestedAmounts()
    }

    var assetBalance: String? {
        guard !assetData.balance.available.isZero else {
            return nil
        }
        return balanceModel.availableBalanceTextWithSymbol
    }

    var fiatProviderViewModel: FiatProvidersViewModel {
        let viewState = viewState
        let selected = selectedQuote(viewState)
        return FiatProvidersViewModel(state: quotesState(viewState).map { items in
            .plain(items.map {
                FiatQuoteViewModel(
                    row: $0,
                    isSelected: $0.provider == selected?.provider,
                    locale: locale,
                )
            })
        })
    }

    var cryptoAmountValue: String {
        guard let selectedQuoteViewModel else { return " " }
        return selectedQuoteViewModel.row.cryptoEstimateText(formattedValue: selectedQuoteViewModel.amountText)
    }

    func providerAssetImage(_ provider: Gemstone.FiatProviderName) -> AssetImage? {
        .image(provider.toPrimitives().image)
    }
}

// MARK: - Actions

extension FiatSceneViewModel {
    func refreshQuotes() async {
        guard session.refreshesQuotes(isScreenActive: true) else { return }
        await load()
    }

    func load() async {
        guard let request = session.quoteRequest() else { return }
        session = session.onFetchStarted(request: request)
        let results: GemFiatQuotesResult
        do {
            let quotes = try await service.quotes(quoteType: request.quoteType, assetId: asset.id.identifier, amount: request.amount)
            results = GemFiatQuotesResult(request: request, quotes: quotes, error: nil)
        } catch let error as GemServiceError {
            guard !error.isCancelled, !Task.isCancelled else { return }
            results = GemFiatQuotesResult(request: request, quotes: [], error: error)
            debugLog("FiatSceneViewModel get quotes error: \(error)")
        } catch {
            debugLog("FiatSceneViewModel get quotes error: \(error)")
            return
        }
        session = session.onQuoteResults(results: results)
    }

    func onAssetDataChange(_: AssetData, _ newValue: AssetData) {
        let type = type
        session = session
            .onBalanceChanged(available: BigUInt(newValue.balance.available))
            .onSellEnabledChanged(isSellEnabled: newValue.metadata.isSellEnabled)
        if session.type != type {
            loadTrigger = FiatLoadTrigger(session: session, isImmediate: true)
        }
    }

    func onSelectContinue() {
        switch viewState.buttonAction {
        case .retryQuote: Task { await load() }
        case .continue: openQuoteUrl()
        }
    }

    func onSelect(amount: Int) {
        applyAmount(String(amount), isImmediate: true)
    }

    func onSelectRandomAmount() {
        applyAmount(String(service.randomAmount()), isImmediate: true)
    }

    func onSelectFiatProviders() {
        isPresentingFiatProvider = true
    }

    func onSelectQuotes(_ quotes: [FiatQuoteViewModel]) {
        guard let quoteModel = quotes.first else { return }
        session = session.onProviderSelected(provider: quoteModel.row.provider)
        isPresentingFiatProvider = false
    }

    func onChangeType(oldType _: FiatQuoteType, newType _: FiatQuoteType) {
        loadTrigger = FiatLoadTrigger(session: session, isImmediate: true)
    }
}

// MARK: - Private

extension FiatSceneViewModel {
    func fiatTransactionsModel() -> FiatTransactionsViewModel {
        FiatTransactionsViewModel(walletId: wallet.id, service: service)
    }

    private var balanceModel: BalanceViewModel {
        BalanceViewModel(asset: asset, balance: assetData.balance, formatter: valueFormatter)
    }

    private var selectedQuoteViewModel: FiatQuoteViewModel? {
        guard let quote = selectedQuote(viewState) else { return nil }
        return FiatQuoteViewModel(row: quote, locale: locale)
    }

    private func applyAmount(_ text: String, isImmediate: Bool) {
        guard text != viewState.amount else { return }
        session = session.onAmountChanged(amount: text)
        loadTrigger = FiatLoadTrigger(session: session, isImmediate: isImmediate)
    }

    private func openQuoteUrl() {
        guard let quote = selectedQuote(viewState) else { return }

        Task {
            urlState = .loading

            do {
                guard let url = try await service.quoteUrl(asset: asset, quoteId: quote.quoteId).redirectUrl.asURL else {
                    urlState = .noData
                    return
                }

                urlState = .data(())
                await UIApplication.shared.open(url, options: [:])
            } catch let error as GemServiceError {
                urlState = .error(error)
                isPresentingAlertMessage = AlertMessage(
                    title: Localized.Errors.errorOccurred,
                    message: error.text().text,
                )
                debugLog("FiatSceneViewModel get quote URL error: \(error)")
            } catch {
                urlState = .error(error)
                debugLog("FiatSceneViewModel get quote URL error: \(error)")
            }
        }
    }
}
