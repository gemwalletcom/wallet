// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemFiatQuoteServiceProtocol
import GemstoneServices
import BigInt
import Components
import Formatters
import Foundation
import GemstonePrimitives
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

    var urlState: StateViewType<Void> = .noData
    var type: FiatQuoteType
    var isPresentingFiatProvider: Bool = false
    var isPresentingAlertMessage: AlertMessage?
    var loadTrigger: FiatLoadTrigger

    let buyViewModel: FiatOperationViewModel
    let sellViewModel: FiatOperationViewModel

    public init(
        service: any GemFiatQuoteServiceProtocol,
        assetAddress: AssetAddress,
        wallet: Wallet,
        type: FiatQuoteType = .buy,
        amount: Int? = nil,
        locale: Locale = .current,
    ) {
        self.service = service
        currencyFormatter = CurrencyFormatter(locale: locale, currencyCode: service.currencyCode)
        self.assetAddress = assetAddress
        self.wallet = wallet
        self.type = type
        assetQuery = ObservableQuery(AssetRequest(walletId: wallet.id, assetId: assetAddress.asset.id), initialValue: .with(asset: assetAddress.asset))
        priceUsdQuery = ObservableQuery(PriceUsdRequest(assetId: assetAddress.asset.id), initialValue: nil)

        buyViewModel = FiatOperationViewModel(service: service, type: .buy, asset: assetAddress.asset, walletId: wallet.id, currencyFormatter: currencyFormatter)
        sellViewModel = FiatOperationViewModel(service: service, type: .sell, asset: assetAddress.asset, walletId: wallet.id, currencyFormatter: currencyFormatter)

        let defaultAmount = switch type {
        case .buy: buyViewModel.amount
        case .sell: sellViewModel.amount
        }

        let initialAmount = amount.map { String($0) } ?? defaultAmount
        loadTrigger = FiatLoadTrigger(type: type, amount: initialAmount, isImmediate: true)

        if let amount {
            currentViewModel.setAmount(String(amount))
        }
    }

    var currentViewModel: FiatOperationViewModel {
        switch type {
        case .buy: buyViewModel
        case .sell: sellViewModel
        }
    }

    var quotesState: StateViewType<[FiatQuote]> {
        currentViewModel.quotesState.map(\.quotes)
    }

    var selectedQuote: FiatQuote? {
        currentViewModel.selectedQuote
    }

    var inputValidationModel: InputValidationViewModel {
        get { currentViewModel.inputValidationModel }
        set { currentViewModel.inputValidationModel = newValue }
    }

    var title: String {
        switch type {
        case .buy: Localized.Buy.title(asset.name)
        case .sell: Localized.Sell.title(asset.name)
        }
    }

    var allowSelectProvider: Bool {
        quotesState.value.or([]).count > 1
    }

    var currencyInputConfig: any CurrencyInputConfigurable {
        FiatCurrencyInputConfig(secondaryText: currentViewModel.cryptoAmountValue, currencySymbol: currencyFormatter.symbol)
    }

    var actionButtonTitle: String {
        Localized.Common.continue
    }

    var actionButtonState: StateViewType<[FiatQuote]> {
        if selectedQuote == nil { return .noData }
        if urlState.isLoading { return .loading }
        if currentViewModel.inputValidationModel.isInvalid || currentViewModel.inputValidationModel.text.isEmptyOrZero { return .noData }
        return quotesState
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
        currentViewModel.emptyTitle
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

    var rateValue: String {
        currentViewModel.rateValue
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
    func load() {
        currentViewModel.load()
    }

    func onAssetDataChange(_: AssetData, _ newValue: AssetData) {
        buyViewModel.onAssetDataChange(newValue)
        sellViewModel.onAssetDataChange(newValue)
    }

    func onSelectContinue() {
        guard let selectedQuote = currentViewModel.selectedQuote else { return }

        Task {
            urlState = .loading

            do {
                guard let url = try await service.quoteUrl(walletId: wallet.id, asset: asset, quoteId: selectedQuote.id).redirectUrl.asURL else {
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

    func onSelect(amount: Int) {
        guard currentViewModel.inputValidationModel.text != String(amount) else { return }
        selectAmount(amount)
    }

    func onSelectRandomAmount() {
        selectAmount(Int(service.randomAmount()))
    }

    func onSelectFiatProviders() {
        isPresentingFiatProvider = true
    }

    func onSelectQuotes(_ quotes: [FiatQuoteViewModel]) {
        guard let quoteModel = quotes.first else { return }
        currentViewModel.selectedQuote = quoteModel.quote
        isPresentingFiatProvider = false
    }

    func onChangeType(oldType: FiatQuoteType, newType: FiatQuoteType) {
        resetStateIfNeeded(for: oldType)
        currentViewModel.setAmount(currentViewModel.amount)
        loadTrigger = FiatLoadTrigger(type: newType, amount: currentViewModel.amount, isImmediate: true)
    }

    func onChangeAmountText(_: String, text: String) {
        guard text != currentViewModel.amount else { return }
        currentViewModel.onChangeAmountText("", text: text)
        loadTrigger = FiatLoadTrigger(type: type, amount: text, isImmediate: false)
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

    private func selectAmount(_ amount: Int) {
        let amountText = String(amount)
        currentViewModel.setAmount(amountText)
        loadTrigger = FiatLoadTrigger(type: type, amount: amountText, isImmediate: true)
    }

    private func resetStateIfNeeded(for type: FiatQuoteType) {
        let model: FiatOperationViewModel = switch type {
        case .buy: buyViewModel
        case .sell: sellViewModel
        }

        switch model.quotesState {
        case .noData, .error: model.quotesState = .loading
        case .loading, .data: break
        }
    }
}
