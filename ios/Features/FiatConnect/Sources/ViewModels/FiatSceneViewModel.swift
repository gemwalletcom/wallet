// Copyright (c). Gem Wallet. All rights reserved.

import BalanceService
import BigInt
import Components
import FiatService
import Formatters
import Foundation
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
    private enum Constants {
        static let randomMaxAmount: Int = 1000
        static let defaultAmount: Int = 50
        static let suggestedAmounts: [Int] = [100, 250]
    }

    let fiatService: FiatService
    private let wallet: Wallet
    private let assetsEnabler: any AssetsEnabler
    private let assetAddress: AssetAddress
    private let currencyFormatter: CurrencyFormatter
    private let valueFormatter = ValueFormatter(locale: .US, style: .medium)

    public let assetQuery: ObservableQuery<AssetRequest>
    var assetData: AssetData {
        assetQuery.value
    }

    var urlState: StateViewType<Void> = .noData
    var type: FiatQuoteType
    var isPresentingFiatProvider: Bool = false
    var isPresentingAlertMessage: AlertMessage?
    var fetchTrigger: FiatFetchTrigger

    let buyViewModel: FiatOperationViewModel
    let sellViewModel: FiatOperationViewModel

    public init(
        fiatService: FiatService,
        currencyFormatter: CurrencyFormatter = CurrencyFormatter(currencyCode: Currency.usd.rawValue),
        assetAddress: AssetAddress,
        wallet: Wallet,
        assetsEnabler: any AssetsEnabler,
        type: FiatQuoteType = .buy,
        amount: Int? = nil,
    ) {
        self.fiatService = fiatService
        self.currencyFormatter = currencyFormatter
        self.assetAddress = assetAddress
        self.wallet = wallet
        self.assetsEnabler = assetsEnabler
        self.type = type
        assetQuery = ObservableQuery(AssetRequest(walletId: wallet.walletId, assetId: assetAddress.asset.id), initialValue: .with(asset: assetAddress.asset))

        let buyOperation = BuyOperation(
            service: fiatService,
            asset: assetAddress.asset,
            currencyFormatter: currencyFormatter,
            walletId: wallet.walletId,
        )
        let sellOperation = SellOperation(
            service: fiatService,
            asset: assetAddress.asset,
            currencyFormatter: currencyFormatter,
            walletId: wallet.walletId,
        )

        buyViewModel = FiatOperationViewModel(
            operation: buyOperation,
            asset: assetAddress.asset,
            currencyFormatter: currencyFormatter,
        )
        sellViewModel = FiatOperationViewModel(
            operation: sellOperation,
            asset: assetAddress.asset,
            currencyFormatter: currencyFormatter,
        )

        let defaultAmount = switch type {
        case .buy: buyViewModel.amount
        case .sell: sellViewModel.amount
        }

        let initialAmount = amount.map { String($0) } ?? defaultAmount
        fetchTrigger = FiatFetchTrigger(type: type, amount: initialAmount, isImmediate: true)

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
        Localized.Errors.errorOccured
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
        Constants.suggestedAmounts
    }

    var showFiatTypePicker: Bool {
        assetData.balance.available > 0 && assetData.metadata.isSellEnabled
    }

    var assetBalance: String? {
        let text = balanceModel.availableBalanceText
        return text == .zero ? nil : text
    }

    var fiatProviderViewModel: FiatProvidersViewModel {
        FiatProvidersViewModel(state: quotesState.map { items in
            .plain(items.map { FiatQuoteViewModel(asset: asset, quote: $0, selectedQuote: selectedQuote, formatter: currencyFormatter) })
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
    func fetch() {
        currentViewModel.fetch()
    }

    func onAssetDataChange(_: AssetData, _ newValue: AssetData) {
        buyViewModel.availableBalance = newValue.balance.available
        sellViewModel.availableBalance = newValue.balance.available
    }

    func onSelectContinue() {
        guard let selectedQuote = currentViewModel.selectedQuote else { return }

        Task {
            urlState = .loading

            do {
                guard let url = try await fiatService.getQuoteUrl(walletId: wallet.walletId.id, quoteId: selectedQuote.id).redirectUrl.asURL else {
                    urlState = .noData
                    return
                }

                urlState = .data(())
                Task { await enableAsset() }
                await UIApplication.shared.open(url, options: [:])
            } catch {
                urlState = .error(error)
                isPresentingAlertMessage = AlertMessage(
                    title: Localized.Errors.errorOccured,
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
        let amount = Int.random(in: Constants.defaultAmount ..< Constants.randomMaxAmount)
        selectAmount(amount)
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
        fetchTrigger = FiatFetchTrigger(type: newType, amount: currentViewModel.amount, isImmediate: true)
    }

    func onChangeAmountText(_: String, text: String) {
        guard text != currentViewModel.amount else { return }
        currentViewModel.onChangeAmountText("", text: text)
        fetchTrigger = FiatFetchTrigger(type: type, amount: text, isImmediate: false)
    }
}

// MARK: - Private

extension FiatSceneViewModel {
    private func enableAsset() async {
        do {
            try await assetsEnabler.enableAssets(wallet: wallet, assetIds: [asset.id], enabled: true)
        } catch {
            debugLog("FiatSceneViewModel enableAsset error: \(error)")
        }
    }

    var walletId: WalletId {
        wallet.walletId
    }

    private var balanceModel: BalanceViewModel {
        BalanceViewModel(asset: asset, balance: assetData.balance, formatter: valueFormatter)
    }

    private func selectAmount(_ amount: Int) {
        let amountText = String(amount)
        currentViewModel.setAmount(amountText)
        fetchTrigger = FiatFetchTrigger(type: type, amount: amountText, isImmediate: true)
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
