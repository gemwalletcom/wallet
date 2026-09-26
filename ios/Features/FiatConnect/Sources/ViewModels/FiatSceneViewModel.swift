// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import func Gemstone.assetListRow
import enum Gemstone.FiatProviderName
import struct Gemstone.GemAssetItemRow
import enum Gemstone.GemFiatAmountCheck
import protocol Gemstone.GemFiatQuoteServiceProtocol
import struct Gemstone.GemFiatQuotesResult
import struct Gemstone.GemFiatSession
import struct Gemstone.GemFiatSuggestedAmount
import struct Gemstone.GemFiatViewState
import struct Gemstone.GemProviderRow
import enum Gemstone.GemSelectAssetType
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

    private let wallet: Wallet
    private let assetAddress: AssetAddress
    private let currencyFormatter: CurrencyFormatter
    private let locale: Locale

    public let priceUsdQuery: ObservableQuery<PriceUsdQuery>
    public let assetQuery: ObservableQuery<AssetQuery>
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
        currencyFormatter = CurrencyFormatter(locale: locale, currencyCode: GemConstants.fiatQuoteCurrency.rawValue)
        self.assetAddress = assetAddress
        self.wallet = wallet
        assetQuery = ObservableQuery(AssetQuery(walletId: wallet.id, assetId: assetAddress.asset.id), initialValue: .with(asset: assetAddress.asset))
        priceUsdQuery = ObservableQuery(PriceUsdQuery(assetId: assetAddress.asset.id), initialValue: nil)
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

    func amountError(_ viewState: GemFiatViewState) -> (any Error)? {
        viewState.amountError.map { AnyError($0.text(locale: locale)) }
    }

    func quotesState(_ viewState: GemFiatViewState) -> StateViewType<[GemProviderRow]> {
        switch viewState.phase {
        case .noInput, .invalidInput, .invalid, .noQuotes: .noData
        case .loading: .loading
        case .ready: .data(viewState.providerRows)
        case let .failed(error): .error(error)
        }
    }

    var title: String {
        switch type {
        case .buy, .sell: type.title(asset: asset.name)
        }
    }

    func currencyInputConfig(_ viewState: GemFiatViewState) -> any CurrencyInputConfigurable {
        FiatCurrencyInputConfig(
            secondaryText: cryptoAmountValue(viewState),
            currencySymbol: currencyFormatter.symbol,
            numberFormat: NumberInput.format(locale),
        )
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
        AssetImage(icon: assetRow.icon)
    }

    var suggestedAmounts: [GemFiatSuggestedAmount] {
        service.suggestedAmounts()
    }

    var assetBalance: String {
        guard case let .value(value, _) = assetRow.trailing else { return .empty }
        return value.text.text
    }

    private var assetRow: GemAssetItemRow {
        assetListRow(
            data: assetData.toGem(),
            currency: GemConstants.fiatQuoteCurrency.toGem(),
            scope: .available,
            style: GemSelectAssetType.buy.flow().rowStyle,
        )
    }

    var fiatProviderViewModel: FiatProvidersViewModel {
        FiatProvidersViewModel(state: quotesState(viewState).map { .plain($0) })
    }

    func cryptoAmountValue(_ viewState: GemFiatViewState) -> String {
        guard let quote = viewState.selectedQuoteRow else { return " " }
        return quote.cryptoEstimateText(formattedValue: quote.cryptoAmount.text(locale: locale))
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

    func onSelectQuotes(_ rows: [GemProviderRow]) {
        guard case let .fiat(provider) = rows.first?.kind else { return }
        session = session.onProviderSelected(provider: provider)
        isPresentingFiatProvider = false
    }

    func onChangeType(oldType _: FiatQuoteType, newType _: FiatQuoteType) {
        loadTrigger = FiatLoadTrigger(session: session, isImmediate: true)
    }
}

// MARK: - Private

extension FiatSceneViewModel {
    func fiatTransactionsModel() -> FiatTransactionsSceneViewModel {
        FiatTransactionsSceneViewModel(walletId: wallet.id, service: service)
    }

    private func applyAmount(_ text: String, isImmediate: Bool) {
        guard text != viewState.amount else { return }
        session = session.onAmountChanged(amount: text)
        loadTrigger = FiatLoadTrigger(session: session, isImmediate: isImmediate)
    }

    private func openQuoteUrl() {
        guard let quote = viewState.selectedQuoteRow else { return }

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
