// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import class Gemstone.Config
import enum Gemstone.GemSwapButtonAction
import struct Gemstone.GemSwapQuotesResult
import struct Gemstone.GemSwapSession
import protocol Gemstone.GemSwapQuoteServiceProtocol
import class Gemstone.GemSwapQuoteSummary
import enum Gemstone.SwapperError
import struct Gemstone.SwapperQuote
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style

@MainActor
@Observable
public final class SwapSceneViewModel {
    static let inputPercentSuggestions = Config.shared.swapConfig().amountPercentPresets.map { PercentageSuggestion(value: Int($0)) }

    public let wallet: Wallet

    public var session: GemSwapSession
    public var isPresentingInfoSheet: SwapSheetType?

    public let fromAssetQuery: ObservableQuery<AssetRequestOptional>
    public let toAssetQuery: ObservableQuery<AssetRequestOptional>

    var fromAsset: AssetData? {
        fromAssetQuery.value
    }

    var toAsset: AssetData? {
        toAssetQuery.value
    }

    // UI states
    var isPresentingPriceImpactConfirmation: String?
    var pairSelectorModel: SwapPairSelectorViewModel

    var selectedSwapQuote: SwapperQuote? {
        session.quote()
    }

    var amountInputModel: InputValidationViewModel = .init(mode: .onDemand)
    var toValue: String = ""
    var loadTrigger: SwapLoadTrigger?

    var quoteDebounce: Duration {
        .milliseconds(service.quoteDebounceMilliseconds())
    }

    var selectedSlippage: SwapSlippage = .auto

    private let onSwap: TransferDataAction
    private let service: any GemSwapQuoteServiceProtocol

    var quoteRefreshInterval: TimeInterval {
        TimeInterval(service.refreshIntervalMilliseconds()) / 1000
    }

    private let formatter = SwapValueFormatter(valueFormatter: .full)
    private let toValueFormatter = SwapValueFormatter(valueFormatter: ValueFormatter(style: .auto))

    public init(
        service: any GemSwapQuoteServiceProtocol,
        input: SwapInput,
        onSwap: TransferDataAction = nil,
    ) {
        let pairSelectorModel = input.pairSelector
        self.pairSelectorModel = pairSelectorModel
        self.service = service
        wallet = input.wallet

        fromAssetQuery = ObservableQuery(AssetRequestOptional(walletId: input.wallet.id, assetId: pairSelectorModel.fromAssetId), initialValue: nil)
        toAssetQuery = ObservableQuery(AssetRequestOptional(walletId: input.wallet.id, assetId: pairSelectorModel.toAssetId), initialValue: nil)
        self.onSwap = onSwap
        selectedSlippage = service.slippage
        session = service.newSession()
    }

    var title: String {
        Localized.Wallet.swap
    }

    var swapFromTitle: String {
        Localized.Swap.youPay
    }

    var swapToTitle: String {
        Localized.Swap.youReceive
    }

    var errorTitle: String {
        Localized.Errors.errorOccurred
    }

    public var swapDetailsViewModel: SwapDetailsViewModel? {
        guard let selectedSwapQuote, let fromAsset, let toAsset else { return nil }
        let summary = GemSwapQuoteSummary.fromQuote(quote: selectedSwapQuote)
        guard let selectedQuote = try? Primitives.SwapQuote(summary.quote()) else { return nil }
        let fromAssetPrice = AssetPriceValue(asset: fromAsset.asset, price: fromAsset.price)
        let toAssetPrice = AssetPriceValue(asset: toAsset.asset, price: toAsset.price)
        return SwapDetailsViewModel(
            state: quotesState.map { providerItems($0, selectedQuote: selectedQuote, toAssetPrice: toAssetPrice) },
            fromAssetPrice: fromAssetPrice,
            toAssetPrice: toAssetPrice,
            selectedQuote: selectedQuote,
            slippage: selectedSlippage,
            currency: service.currency.rawValue,
            isProviderSelectionEnabled: isQuoteInteractionEnabled,
            swapPriceImpact: fromAssetPrice.swapValue(BigUInt(selectedQuote.fromValueBigInt))
                .priceImpact(receive: toAssetPrice.swapValue(BigUInt(selectedQuote.toValueBigInt)))
                .map { $0.map() },
            minReceiveValue: BigInt(summary.minReceiveValue()),
            etaMinutes: summary.etaMinutes(),
            swapProviderSelectAction: { [weak self] quote in
                self?.onFinishSwapProviderSelection(quote)
            },
        )
    }

    private func providerItems(_ quotes: [SwapperQuote], selectedQuote: SwapQuote, toAssetPrice: AssetPriceValue) -> [SwapProviderItem] {
        quotes.compactMap {
            SwapProviderItem(
                asset: toAssetPrice.asset,
                swapperQuote: $0,
                selectedProvider: selectedQuote.providerData.provider,
                priceViewModel: PriceViewModel(price: toAssetPrice.price, currencyCode: service.currency.rawValue),
                valueFormatter: ValueFormatter(style: .auto),
            )
        }
    }

    var showsSlippageIndicator: Bool {
        selectedSlippage.isCustom
    }

    var swapSlippageViewModel: SwapSlippageViewModel? {
        guard let fromAsset else { return nil }
        return SwapSlippageViewModel(
            service: service,
            chain: fromAsset.asset.chain,
            slippage: selectedSlippage,
            onSelect: { [weak self] slippage in
                self?.onSelectSlippage(slippage)
            },
        )
    }

    var buttonViewModel: SwapButtonViewModel {
        SwapButtonViewModel(
            session: session,
            buttonAction: buttonAction,
            fromAsset: fromAsset,
            onAction: onSelectActionButton,
        )
    }

    var shouldShowAdditionalInfo: Bool {
        !session.isQuoteLoading()
    }

    var isQuoteLoading: Bool {
        session.isQuoteLoading()
    }

    var isTransferDataLoading: Bool {
        session.isTransferLoading()
    }

    var error: (any Error)? {
        session.transferError() ?? session.quoteError()
    }

    var isQuoteInteractionEnabled: Bool {
        !isTransferDataLoading
    }

    var isReceiveFieldLoading: Bool {
        isQuoteLoading
    }

    var assetIds: Set<AssetId> {
        Set([fromAsset?.asset.id, toAsset?.asset.id].compactMap(\.self))
    }

    var errorInfoAction: VoidAction {
        guard let error = session.quoteError(), case .NoQuoteAvailable = error else {
            return nil
        }
        return VoidAction { [weak self] in
            self?.isPresentingInfoSheet = .info(.noQuote)
        }
    }

    private var payTokenInteraction: SwapTokenInteraction {
        .pay(isEnabled: isQuoteInteractionEnabled)
    }

    private var receiveTokenInteraction: SwapTokenInteraction {
        .receive(isEnabled: isQuoteInteractionEnabled)
    }

    func swapTokenModel(type: SelectAssetSwapType) -> SwapTokenViewModel {
        let interaction = switch type {
        case .pay: payTokenInteraction
        case .receive: receiveTokenInteraction
        }
        guard let assetData: AssetData = type == .pay ? fromAsset : toAsset else {
            return SwapTokenViewModel(
                type: .placeholder,
                interaction: interaction,
            )
        }
        return SwapTokenViewModel(
            type: .selected(
                AssetDataViewModel(
                    assetData: assetData,
                    formatter: .auto,
                    currencyCode: service.currency.rawValue,
                    currencyFormatterType: .currency,
                ),
            ),
            interaction: interaction,
        )
    }
}

// MARK: - Business Logic

extension SwapSceneViewModel {
    func suggestPair() async {
        guard
            pairSelectorModel.toAssetId == nil,
            let pair = try? await service.suggestPair(payAssetId: pairSelectorModel.fromAssetId?.identifier)?.map()
        else { return }
        pairSelectorModel = pair
    }

    func load() async {
        guard !isTransferDataLoading, !session.refreshPausedUntilRestart, let currentInput else { return }
        await performFetch(input: currentInput)
    }

    func onAppear() {
        session = session.onRefreshResumed()
    }

    func onChangePair(_ _: SwapPairSelectorViewModel, _ newModel: SwapPairSelectorViewModel) {
        fromAssetQuery.request.assetId = newModel.fromAssetId
        toAssetQuery.request.assetId = newModel.toAssetId
    }

    func onChangeSwapQuote(_ _: SwapperQuote?, _ newQuote: SwapperQuote?) {
        guard !isTransferDataLoading, let newQuote, let toAsset else { return }
        applyQuote(newQuote, asset: toAsset.asset)
    }

    func onChangeFromValue(_: String, _: String) {
        if let input = loadTrigger?.input, input == currentInput {
            return
        }
        setLoadTrigger(isImmediate: false)
    }

    func onChangeFromAsset(old: AssetData?, new: AssetData?) {
        guard old?.asset.id != new?.asset.id else { return }

        resetValues()
        setLoadTrigger(isImmediate: true)
    }

    func onChangeToAsset(old: AssetData?, new: AssetData?) {
        guard old?.asset.id != new?.asset.id else { return }

        resetToValue()
        setLoadTrigger(isImmediate: true)
    }

    func onSelectFromMaxBalance() {
        onSelectPercent(100)
    }

    func onSelectPercent(_ percent: Int) {
        guard let fromAsset else { return }
        applyPercentToFromValue(percent: percent, assetData: fromAsset)
        setLoadTrigger(isImmediate: true)
    }

    func onSelectSwapConfirmation() {
        swap()
    }

    func onAssetIdsChange(assetIds: Set<AssetId>) async {
        await performUpdate(for: Array(assetIds))
    }

    func onSelectAssetPay() {
        isPresentingInfoSheet = .selectAsset(.pay)
    }

    func onSelectAssetReceive() {
        guard let fromAsset else { return }
        let (chains, assetIds) = service.supportedAssets(for: fromAsset.asset.id)
        isPresentingInfoSheet = .selectAsset(.receive(chains: chains, assetIds: assetIds))
    }

    func onSelectSwapDetails() {
        isPresentingInfoSheet = .swapDetails
    }

    func onFinishSwapProviderSelection(_ quote: SwapperQuote) {
        session = session.onProviderSelected(provider: quote.data.provider.id)
    }

    func onSelectSlippage(_ slippage: SwapSlippage) {
        guard slippage != selectedSlippage else { return }
        selectedSlippage = slippage
        do {
            try service.setSlippage(slippage)
        } catch {
            debugLog("set swap slippage error: \(error)")
        }
        setLoadTrigger(isImmediate: true)
    }

    public func onFinishAssetSelection(asset: Asset) {
        guard case let .selectAsset(type) = isPresentingInfoSheet else { return }
        switch type {
        case .pay:
            if asset.id == pairSelectorModel.toAssetId {
                pairSelectorModel.toAssetId = pairSelectorModel.fromAssetId
            }
            pairSelectorModel.fromAssetId = asset.id
        case .receive:
            if asset.id == pairSelectorModel.fromAssetId {
                pairSelectorModel.fromAssetId = pairSelectorModel.toAssetId
            }
            pairSelectorModel.toAssetId = asset.id
        }
        isPresentingInfoSheet = nil
    }
}

// MARK: - Private

extension SwapSceneViewModel {
    private var buttonAction: GemSwapButtonAction {
        session.buttonAction(value: currentInput?.value ?? .zero, availableBalance: fromAsset?.balance.available ?? .zero)
    }

    private var quotesState: StateViewType<[SwapperQuote]> {
        if session.isQuoteLoading() {
            return .loading
        }
        if let error = session.quoteError() {
            return .error(error)
        }
        if let quotes = session.quotes?.quotes {
            return .data(quotes)
        }
        return .noData
    }

    private var currentInput: SwapQuoteInput? {
        try? SwapQuoteInput.create(
            fromAsset: fromAsset,
            toAsset: toAsset,
            fromValue: amountInputModel.text,
            slippage: selectedSlippage,
            formatter: formatter,
        )
    }

    private func resetValues() {
        resetToValue()
        amountInputModel.text = .empty
    }

    private func resetToValue() {
        toValue = ""
    }

    private func applyQuote(_ quote: SwapperQuote, asset: Asset) {
        toValue = toValueFormatter.format(value: BigInt(quote.toValue), decimals: asset.decimals.asInt)
    }

    private func applyPercentToFromValue(percent: Int, assetData: AssetData) {
        amountInputModel.text = formatter.format(
            value: assetData.balance.available.multiply(byPercent: percent),
            decimals: assetData.asset.decimals.asInt,
        )
    }

    private func applyMinAmount(_ value: BigInt) {
        guard let fromAsset else { return }
        amountInputModel.text = formatter.format(value: value, decimals: fromAsset.asset.decimals.asInt)
        setLoadTrigger(isImmediate: true)
    }

    private func setLoadTrigger(isImmediate: Bool) {
        guard let input = currentInput else {
            resetToValue()
            session = session.onRequestChanged(request: nil)
            loadTrigger = nil
            return
        }
        guard !isTransferDataLoading else { return }
        session = session.onRequestChanged(request: input.request)
        loadTrigger = SwapLoadTrigger(input: input, isImmediate: isImmediate)

        Task {
            let assetIds = [fromAsset?.asset.id, toAsset?.asset.id].compactMap(\.self)
            try await service.addPrices(assetIds: assetIds)
        }
    }

    private func swap() {
        guard let fromAsset, let toAsset, let started = session.startTransfer(), let quote = started.quote() else {
            return
        }
        let transfer = started.transferPhase
        session = started

        Task {
            do {
                let transferData = try await service.getTransferData(
                    fromAsset: fromAsset.asset,
                    toAsset: toAsset.asset,
                    quote: quote,
                )
                guard session.transferPhase == transfer else { return }
                onSwap?(transferData)
                session = session.onTransferHandedOff(transfer: transfer)
            } catch {
                session = session.onTransferFailed(transfer: transfer, error: error.swapperError ?? .ComputeQuoteError(error.localizedDescription))
                debugLog("SwapScene get swap data error: \(error)")
            }
        }
    }

    private func performFetch(input: SwapQuoteInput) async {
        guard !isTransferDataLoading else { return }
        session = session.onFetchStarted(request: input.request)
        resetToValue()
        do {
            let swapQuotes = try await service.getQuotes(
                fromAsset: input.fromAsset,
                toAsset: input.toAsset,
                amount: input.value,
                useMaxAmount: input.useMaxAmount,
                slippage: input.slippage,
            )

            guard currentInput == input else { return }
            session = session.onQuoteResults(results: GemSwapQuotesResult(request: input.request, quotes: swapQuotes, error: nil))
            if let selectedSwapQuote, let asset = toAsset?.asset {
                applyQuote(selectedSwapQuote, asset: asset)
            }
        } catch {
            if !error.isCancelled, !Task.isCancelled {
                guard currentInput == input else { return }
                let failure = error.swapperError ?? .ComputeQuoteError(error.localizedDescription)
                session = session.onQuoteResults(results: GemSwapQuotesResult(request: input.request, quotes: [], error: failure))
                debugLog("SwapScene get quotes error: \(error)")
            }
        }
    }

    private func performUpdate(for assetIds: [AssetId]) async {
        do {
            try await service.updateBalances(assetIds: assetIds)
        } catch {
            debugLog("SwapScene balance update error: \(error)")
        }
    }

    private func onSelectActionButton() {
        switch buttonAction {
        case .retryQuote: setLoadTrigger(isImmediate: true)
        case .retryTransfer: swap()
        case .insufficientBalance: break
        case let .useMinimumAmount(value): applyMinAmount(value)
        case .swap:
            if let priceImpactModel = swapDetailsViewModel?.priceImpactModel,
               let warningText = priceImpactModel.highImpactWarningDescription,
               priceImpactModel.showPriceImpactWarning
            {
                isPresentingPriceImpactConfirmation = warningText
                return
            }
            swap()
        }
    }
}

extension Error {
    var swapperError: Gemstone.SwapperError? {
        self as? Gemstone.SwapperError
    }
}
