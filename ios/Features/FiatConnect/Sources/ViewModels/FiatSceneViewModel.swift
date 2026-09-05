// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import struct Gemstone.GemFiatQuotesResult
import struct Gemstone.GemFiatSession
import protocol Gemstone.GemFiatQuoteServiceProtocol
import enum Gemstone.GemServiceError
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI
import Validators

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
    var loadTrigger: FiatLoadTrigger
    var inputValidationModel = InputValidationViewModel(mode: .onDemand)

    public init(
        service: any GemFiatQuoteServiceProtocol,
        assetAddress: AssetAddress,
        wallet: Wallet,
        type: FiatQuoteType = .buy,
        amount: Int? = nil,
        locale: Locale = .current,
    ) {
        self.service = service
        currencyFormatter = CurrencyFormatter(locale: locale, currencyCode: service.currency.rawValue)
        self.assetAddress = assetAddress
        self.wallet = wallet
        assetQuery = ObservableQuery(AssetRequest(walletId: wallet.id, assetId: assetAddress.asset.id), initialValue: .with(asset: assetAddress.asset))
        priceUsdQuery = ObservableQuery(PriceUsdRequest(assetId: assetAddress.asset.id), initialValue: nil)
        let session = service.newSession(type: type, amount: amount)
        self.session = session
        loadTrigger = FiatLoadTrigger(type: type, amount: session.amount, isImmediate: true)
        inputValidationModel.text = session.amount
        updateValidators()
    }

    var type: FiatQuoteType {
        get { session.type }
        set { session = session.onTypeChanged(quoteType: newValue.map()) }
    }

    var quotesState: StateViewType<[FiatQuote]> {
        switch session.current().phase {
        case .noInput, .invalidInput, .invalid, .noQuotes: .noData
        case .loading: .loading
        case .ready: .data(session.fiatQuotes)
        case let .failed(error): .error(error)
        }
    }

    var selectedQuote: FiatQuote? {
        session.selectedFiatQuote
    }

    var title: String {
        switch type {
        case .buy: Localized.Buy.title(asset.name)
        case .sell: Localized.Sell.title(asset.name)
        }
    }

    var allowSelectProvider: Bool {
        session.canSelectProvider()
    }

    var currencyInputConfig: any CurrencyInputConfigurable {
        FiatCurrencyInputConfig(secondaryText: cryptoAmountValue, currencySymbol: currencyFormatter.symbol)
    }

    var actionButtonTitle: String {
        switch session.buttonAction() {
        case .continue: Localized.Common.continue
        case .retryQuote: Localized.Common.tryAgain
        }
    }

    var actionButtonState: ButtonState {
        switch session.buttonState(isUrlLoading: urlState.isLoading) {
        case .disabled: .disabled
        case .loading: .loading(showProgress: true)
        case .enabled: .normal
        }
    }

    var providerTitle: String {
        Localized.Common.provider
    }

    var rateTitle: String {
        Localized.Buy.rate
    }

    var errorTitle: String {
        Localized.Errors.errorOccurred
    }

    var emptyTitle: String {
        switch session.current().phase {
        case .noInput, .invalidInput:
            switch type {
            case .buy: Localized.Input.enterAmountTo(Localized.Wallet.buy)
            case .sell: Localized.Input.enterAmountTo(Localized.Wallet.sell)
            }
        case .invalid, .loading, .ready, .noQuotes, .failed: Localized.Buy.noResults
        }
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

    var suggestedAmounts: [Int] {
        service.config().suggestedAmounts.map(Int.init)
    }

    var showFiatTypePicker: Bool {
        assetData.metadata.isSellEnabled
    }

    var assetBalance: String? {
        guard !assetData.balance.available.isZero else {
            return nil
        }
        return balanceModel.availableBalanceTextWithSymbol
    }

    var fiatProviderViewModel: FiatProvidersViewModel {
        FiatProvidersViewModel(state: quotesState.map { items in
            .plain(items.map {
                FiatQuoteViewModel(
                    asset: asset,
                    quote: $0,
                    assetPrice: priceUsdQuery.value,
                    isSelected: $0.provider == selectedQuote?.provider,
                    formatter: currencyFormatter,
                )
            })
        })
    }

    var cryptoAmountValue: String {
        guard let selectedQuoteViewModel else { return " " }
        return "≈ \(selectedQuoteViewModel.amountText)"
    }

    var rateValue: String {
        guard let selectedQuoteViewModel else { return "" }
        return "1 \(asset.symbol) ≈ \(selectedQuoteViewModel.rateText)"
    }

    func buttonTitle(amount: Int) -> String {
        "\(currencyFormatter.symbol)\(amount)"
    }

    func providerAssetImage(_ provider: FiatProvider) -> AssetImage? {
        .image(provider.image)
    }
}

// MARK: - Actions

extension FiatSceneViewModel {
    func load() async {
        guard let request = session.quoteRequest() else { return }
        session = session.onFetchStarted(request: request)
        let results: GemFiatQuotesResult
        do {
            let quotes = try await service.quotes(quoteType: request.quoteType, assetId: asset.id.identifier, amount: request.amount)
            results = GemFiatQuotesResult(request: request, quotes: quotes, error: nil)
        } catch {
            guard !error.isCancelled, !Task.isCancelled else { return }
            results = GemFiatQuotesResult(request: request, quotes: [], error: error as? GemServiceError ?? .Core(msg: error.localizedDescription))
            debugLog("FiatSceneViewModel get quotes error: \(error)")
        }
        session = session.onQuoteResults(results: results)
        updateValidators()
    }

    func onAssetDataChange(_: AssetData, _ newValue: AssetData) {
        let type = type
        session = session
            .onBalanceChanged(available: BigUInt(newValue.balance.available))
            .onSellEnabledChanged(isSellEnabled: newValue.metadata.isSellEnabled)
        if session.type != type {
            applyAmount(session.amount, isImmediate: true)
        }
        updateValidators()
    }

    func onSelectContinue() {
        switch session.buttonAction() {
        case .retryQuote: Task { await load() }
        case .continue: openQuoteUrl()
        }
    }

    func onSelect(amount: Int) {
        guard inputValidationModel.text != String(amount) else { return }
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
        session = session.onProviderSelected(provider: quoteModel.quote.provider.id)
        updateValidators()
        isPresentingFiatProvider = false
    }

    func onChangeType(oldType _: FiatQuoteType, newType: FiatQuoteType) {
        inputValidationModel.text = session.amount
        updateValidators()
        loadTrigger = FiatLoadTrigger(type: newType, amount: session.amount, isImmediate: true)
    }

    func onChangeAmountText(_: String, text: String) {
        guard text != session.amount else { return }
        applyAmount(text, isImmediate: false)
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
        guard let selectedQuote else { return nil }
        return FiatQuoteViewModel(asset: asset, quote: selectedQuote, formatter: currencyFormatter)
    }

    private func applyAmount(_ text: String, isImmediate: Bool) {
        session = session.onAmountChanged(amount: text)
        inputValidationModel.text = text
        updateValidators()
        loadTrigger = FiatLoadTrigger(type: type, amount: text, isImmediate: isImmediate)
    }

    private func updateValidators() {
        let validator = FiatAmountValidator(
            service: service,
            type: type,
            asset: asset,
            quote: selectedQuote,
            availableBalance: assetData.balance.available,
            currencyFormatter: currencyFormatter,
        )
        inputValidationModel.update(validators: [.assetAmount(decimals: 0, validators: [validator])])
    }

    private func openQuoteUrl() {
        guard let selectedQuote else { return }

        Task {
            urlState = .loading

            do {
                guard let url = try await service.quoteUrl(asset: asset, quoteId: selectedQuote.id).redirectUrl.asURL else {
                    urlState = .noData
                    return
                }

                urlState = .data(())
                await UIApplication.shared.open(url, options: [:])
            } catch {
                urlState = .error(error)
                isPresentingAlertMessage = AlertMessage(
                    title: Localized.Errors.errorOccurred,
                    message: error.localizedDescription,
                )
                debugLog("FiatSceneViewModel get quote URL error: \(error)")
            }
        }
    }
}
