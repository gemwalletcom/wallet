// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Foundation
import func Gemstone.assetListRow
import enum Gemstone.FiatProviderName
import struct Gemstone.GemAssetItemRow
import enum Gemstone.GemFiatAmountCheck
import protocol Gemstone.GemFiatQuoteServiceProtocol
import struct Gemstone.GemFiatSession
import struct Gemstone.GemFiatSuggestedAmount
import struct Gemstone.GemFiatViewState
import enum Gemstone.GemInfoTopic
import struct Gemstone.GemProviderRow
import enum Gemstone.GemSelectAssetType
import GemstonePrimitives
import GemstoneServices
import InfoSheet
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

    let priceUsdQuery: ObservableQuery<PriceUsdQuery>
    public let assetQuery: ObservableQuery<AssetQuery>
    var assetData: AssetData {
        assetQuery.value
    }

    var session: GemFiatSession
    var isUrlLoading = false
    var isPresentingFiatProvider: Bool = false
    var isPresentingInfoSheet: InfoSheetModel?
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
        session = service.newSession(quoteType: type.toGem(), amount: amount.map { UInt32($0) }, format: NumberInput.format(locale))
        loadTrigger = FiatLoadTrigger(session: session, isImmediate: true)
    }

    var type: FiatQuoteType {
        get { session.quoteType.toPrimitives() }
        set { session = session.onTypeChanged(quoteType: newValue.toGem()) }
    }

    var viewState: GemFiatViewState {
        session.viewState(assetPrice: priceUsdQuery.value, isUrlLoading: isUrlLoading, isSellEnabled: assetData.metadata.isSellEnabled)
    }

    var amount: String {
        get { viewState.amount }
        set { setAmount(newValue, isImmediate: false) }
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

    var fiatProviderViewModel: ProvidersViewModel {
        ProvidersViewModel(state: quotesState(viewState).map { .plain($0) })
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
        let results = await service.quotes(request: request, assetId: asset.id)
        guard results.error == nil || !Task.isCancelled else { return }
        session = session.onQuoteResults(results: results)
    }

    func onAssetDataChange(_: AssetData, _ newValue: AssetData) {
        let type = type
        session = session
            .onBalanceChanged(available: BigUInt(newValue.balance.available))
            .onSellEnabledChanged(isSellEnabled: newValue.metadata.isSellEnabled)
        if session.quoteType.toPrimitives() != type {
            loadTrigger = FiatLoadTrigger(session: session, isImmediate: true)
        }
    }

    func onSelectContinue() async {
        switch viewState.buttonAction {
        case .retryQuote: await load()
        case .continue: await openQuoteUrl()
        }
    }

    func onSelect(amount: Int) {
        setAmount(String(amount), isImmediate: true)
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

    private func setAmount(_ text: String, isImmediate: Bool) {
        guard text != viewState.amount else { return }
        session = session.onAmountChanged(amount: text)
        loadTrigger = FiatLoadTrigger(session: session, isImmediate: isImmediate)
    }

    private func openQuoteUrl() async {
        guard !isUrlLoading, let quote = viewState.selectedQuoteRow, let request = session.quoteRequest() else { return }
        guard service.isAvailable(quoteType: request.quoteType) else {
            isPresentingInfoSheet = InfoSheetModel(sheet: GemInfoTopic.regionUnavailable.infoSheet)
            return
        }
        isUrlLoading = true
        defer { isUrlLoading = false }
        do {
            guard let url = try await service.quoteUrl(assetId: asset.id, quoteId: quote.quoteId).redirectUrl.asURL else { return }
            guard session.quoteRequest() == request else { return }
            await UIApplication.shared.open(url, options: [:])
        } catch {
            isPresentingInfoSheet = InfoSheetModel(error: error)
        }
    }
}
