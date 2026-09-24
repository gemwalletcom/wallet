// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import class Gemstone.Config
import func Gemstone.formattedPercentage
import enum Gemstone.GemSlippageSelection
import enum Gemstone.GemSwapButtonAction
import enum Gemstone.GemSwapErrorDisplay
import struct Gemstone.GemSwapPairSelection
import struct Gemstone.GemSwapQuoteInput
import protocol Gemstone.GemSwapQuoteServiceProtocol
import struct Gemstone.GemSwapQuotesResult
import struct Gemstone.GemSwapSession
import enum Gemstone.GemSwapSide
import struct Gemstone.GemSwapViewState
import enum Gemstone.SwapperError
import struct Gemstone.SwapperQuote
import func Gemstone.swapperQuoteSummary
import enum Gemstone.SwapProvider
import struct Gemstone.SwapQuote
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style

@MainActor
@Observable
public final class SwapSceneViewModel {
    static let inputPercentSuggestions = Config.shared.swapConfig().amountPercentPresets.map {
        PercentageSuggestion(number: formattedPercentage(value: Double($0), style: .unsignedCompact))
    }

    public let wallet: Wallet

    public var session: GemSwapSession
    @ObservationIgnored private var viewStateCache: (session: GemSwapSession, availableBalance: BigInt, state: GemSwapViewState)?
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

    var viewState: GemSwapViewState {
        let availableBalance = fromAsset?.balance.available ?? .zero
        if let cache = viewStateCache, cache.session == session, cache.availableBalance == availableBalance {
            return cache.state
        }
        let state = session.viewState(availableBalance: availableBalance, payAsset: fromAsset?.asset.toGem())
        viewStateCache = (session, availableBalance, state)
        return state
    }

    var selectedSwapQuote: SwapperQuote? {
        viewState.quote
    }

    var amountInputModel: InputValidationViewModel = .init(mode: .onDemand)
    var toValue: String = ""
    var loadTrigger: SwapLoadTrigger?

    var quoteDebounce: Duration {
        .milliseconds(service.quoteDebounceMilliseconds())
    }

    var selectedSlippage: GemSlippageSelection = .auto

    private let onSwap: TransferDataAction
    private let service: any GemSwapQuoteServiceProtocol

    var quoteRefreshInterval: TimeInterval {
        TimeInterval(service.refreshIntervalMilliseconds()) / 1000
    }

    private let toValueFormatter = ValueFormatter(style: .auto)

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
        let summary = swapperQuoteSummary(quote: selectedSwapQuote, fromAsset: fromAsset.asset.toGem(), toAsset: toAsset.asset.toGem())
        let selectedQuote = summary.quote
        let fromAssetPrice = AssetPriceValue(asset: fromAsset.asset, price: fromAsset.price)
        let toAssetPrice = AssetPriceValue(asset: toAsset.asset, price: toAsset.price)
        return SwapDetailsViewModel(
            state: quotesState.map { _ in providerItems(toAssetPrice: toAssetPrice) },
            fromAssetPrice: fromAssetPrice,
            toAssetPrice: toAssetPrice,
            summary: summary,
            slippagePercent: selectedSlippage.bps.map { service.slippagePercent(bps: $0) },
            currency: service.currency.rawValue,
            allowSelectProvider: viewState.allowsProviderSelection,
            swapPriceImpact: fromAssetPrice.swapValue(selectedQuote.fromValue)
                .priceImpact(receive: toAssetPrice.swapValue(selectedQuote.toValue)),
            swapProviderSelectAction: { [weak self] provider in
                self?.onFinishSwapProviderSelection(provider)
            },
        )
    }

    private func providerItems(toAssetPrice: AssetPriceValue) -> [SwapProviderItem] {
        let rows = session.providerRows(
            receiveAsset: toAssetPrice.asset.toGem(),
            receivePrice: toAssetPrice.price?.price,
            currency: service.currency.toGem(),
        )
        return rows.map(SwapProviderItem.init(row:))
    }

    var showsSlippageIndicator: Bool {
        selectedSlippage.isCustom
    }

    var swapSlippageViewModel: SwapSlippageViewModel? {
        guard let fromAsset else { return nil }
        return SwapSlippageViewModel(
            chain: fromAsset.asset.chain,
            slippage: selectedSlippage,
            onSelect: { [weak self] slippage in
                self?.onSelectSlippage(slippage)
            },
        )
    }

    var buttonViewModel: SwapButtonViewModel {
        SwapButtonViewModel(
            state: viewState,
            fromAsset: fromAsset,
            onAction: onSelectActionButton,
        )
    }

    var shouldShowAdditionalInfo: Bool {
        !viewState.isQuoteLoading
    }

    var isQuoteLoading: Bool {
        viewState.isQuoteLoading
    }

    var isTransferDataLoading: Bool {
        viewState.isTransferLoading
    }

    var error: GemSwapErrorDisplay? {
        viewState.error
    }

    var isReceiveFieldLoading: Bool {
        viewState.isReceiveLoading
    }

    var assetIds: Set<AssetId> {
        Set([fromAsset?.asset.id, toAsset?.asset.id].compactMap(\.self))
    }

    var errorInfoAction: VoidAction {
        guard let topic = viewState.error?.info() else {
            return nil
        }
        return VoidAction { [weak self] in
            self?.isPresentingInfoSheet = .info(InfoSheetType(topic: topic, assetImage: nil))
        }
    }

    func swapTokenModel(type: SelectAssetSwapType) -> SwapTokenViewModel {
        let interaction = switch type {
        case .pay: viewState.pay
        case .receive: viewState.receive
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
                    currency: service.currency,
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
            let pair = await service.suggestPair(payAssetId: pairSelectorModel.fromAssetId?.identifier)?.map()
        else { return }
        pairSelectorModel = pair
    }

    func load() async {
        guard session.refreshesQuotes(isScreenActive: true), let input = session.input else { return }
        await fetchQuotes(input: input)
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
        setToValue(quote: newQuote, asset: toAsset.asset)
    }

    func onChangeFromValue(_: String, _: String) {
        updateSessionInput()
        if loadTrigger?.input == session.input {
            return
        }
        setLoadTrigger(isImmediate: false)
    }

    func onChangeFromAsset(old: AssetData?, new: AssetData?) {
        guard old?.asset.id != new?.asset.id else { return }

        resetValues()
        updateSessionInput(amount: "")
        setLoadTrigger(isImmediate: true)
    }

    func onChangeToAsset(old: AssetData?, new: AssetData?) {
        guard old?.asset.id != new?.asset.id else { return }

        resetToValue()
        updateSessionInput()
        setLoadTrigger(isImmediate: true)
    }

    func onSelectFromMaxBalance() {
        onSelectPercent(100)
    }

    func onSelectPercent(_ percent: Int) {
        guard let fromAsset else { return }
        setFromValue(percent: percent, assetData: fromAsset)
        updateSessionInput()
        setLoadTrigger(isImmediate: true)
    }

    func onSelectSwapConfirmation() {
        swap()
    }

    func onAssetIdsChange(assetIds: Set<AssetId>) async {
        for failure in await service.refreshPair(assetIds: Array(assetIds)) {
            debugLog("SwapScene pair refresh error: \(failure.step) \(failure.message)")
        }
    }

    func onSelectAssetPay() {
        isPresentingInfoSheet = .selectAsset(.pay)
    }

    func onSelectAssetReceive() {
        guard let fromAsset else { return }
        isPresentingInfoSheet = .selectAsset(.receive(payAssetId: fromAsset.asset.id))
    }

    func onSelectSwapDetails() {
        isPresentingInfoSheet = .swapDetails
    }

    func onFinishSwapProviderSelection(_ provider: SwapProvider) {
        session = session.onProviderSelected(provider: provider)
    }

    func onSelectSlippage(_ slippage: GemSlippageSelection) {
        guard slippage != selectedSlippage else { return }
        selectedSlippage = slippage
        do {
            try service.setSlippage(slippage)
        } catch {
            debugLog("set swap slippage error: \(error)")
        }
        updateSessionInput()
        setLoadTrigger(isImmediate: true)
    }

    public func onFinishAssetSelection(asset: Asset) {
        guard case let .selectAsset(type) = isPresentingInfoSheet else { return }
        let side: GemSwapSide = switch type {
        case .pay: .pay
        case .receive: .receive
        }
        let selection = service.selectPairAsset(
            selection: GemSwapPairSelection(
                payAssetId: pairSelectorModel.fromAssetId?.identifier,
                receiveAssetId: pairSelectorModel.toAssetId?.identifier,
            ),
            side: side,
            assetId: asset.id.identifier,
        )
        pairSelectorModel.fromAssetId = selection.payAssetId.map { AssetId(core: $0) }
        pairSelectorModel.toAssetId = selection.receiveAssetId.map { AssetId(core: $0) }
        isPresentingInfoSheet = nil
    }
}

// MARK: - Private

extension SwapSceneViewModel {
    private var quotesState: StateViewType<[SwapperQuote]> {
        switch viewState.quotesState {
        case .loading: .loading
        case let .failed(error): .error(error)
        case .quotes: .data(session.quotes?.quotes ?? [])
        case .empty: .noData
        }
    }

    private var selectedSlippageBps: UInt32? {
        switch selectedSlippage {
        case .auto: nil
        case let .manual(bps): bps
        }
    }

    private func updateSessionInput(amount: String? = nil) {
        session = session.onInputChanged(
            amount: amount ?? amountInputModel.text,
            payAsset: fromAsset?.asset.toGem(),
            receiveAsset: toAsset?.asset.toGem(),
            availableValue: fromAsset?.balance.available ?? .zero,
            slippageBps: selectedSlippageBps,
            format: NumberInput.format(),
        )
    }

    private func resetValues() {
        resetToValue()
        amountInputModel.text = .empty
    }

    private func resetToValue() {
        toValue = ""
    }

    private func setToValue(quote: SwapperQuote, asset: Asset) {
        toValue = toValueFormatter.string(BigInt(quote.toValue), decimals: asset.decimals.asInt)
    }

    private func setFromValue(percent: Int, assetData: AssetData) {
        let value = service.amountForPercent(available: assetData.balance.available, percent: UInt32(percent))
        guard let text = NumberInput.format().inputText(value: value.description, decimals: UInt32(assetData.asset.decimals)) else { return }
        amountInputModel.text = text
    }

    private func setFromValue(minimum value: BigInt) {
        guard let fromAsset, let text = NumberInput.format().inputText(value: value.description, decimals: UInt32(fromAsset.asset.decimals)) else { return }
        amountInputModel.text = text
        updateSessionInput()
        setLoadTrigger(isImmediate: true)
    }

    private func setLoadTrigger(isImmediate: Bool) {
        guard let input = session.input else {
            resetToValue()
            loadTrigger = nil
            return
        }
        guard !isTransferDataLoading else { return }
        resetToValue()
        loadTrigger = SwapLoadTrigger(input: input, isImmediate: isImmediate)
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
            } catch let error as SwapperError {
                session = session.onTransferFailed(transfer: transfer, error: error)
                debugLog("SwapScene get swap data error: \(error)")
            } catch {
                debugLog("SwapScene get swap data error: \(error)")
            }
        }
    }

    private func fetchQuotes(input: GemSwapQuoteInput) async {
        guard
            !isTransferDataLoading,
            let fromAsset, fromAsset.asset.id.identifier == input.request.payAssetId,
            let toAsset, toAsset.asset.id.identifier == input.request.receiveAssetId
        else { return }
        session = session.onFetchStarted(request: input.request)
        resetToValue()
        do {
            let swapQuotes = try await service.getQuotes(fromAsset: fromAsset.asset, toAsset: toAsset.asset, input: input)
            session = session.onQuoteResults(results: GemSwapQuotesResult(request: input.request, quotes: swapQuotes, error: nil))
            if let selectedSwapQuote {
                setToValue(quote: selectedSwapQuote, asset: toAsset.asset)
            }
        } catch let error as SwapperError {
            guard !Task.isCancelled else { return }
            session = session.onQuoteResults(results: GemSwapQuotesResult(request: input.request, quotes: [], error: error))
            debugLog("SwapScene get quotes error: \(error)")
        } catch {
            debugLog("SwapScene get quotes error: \(error)")
        }
    }

    private func onSelectActionButton() {
        switch viewState.buttonAction {
        case .retryQuote:
            if let input = session.input {
                session = session.onRefreshRequested(request: input.request)
            }
            setLoadTrigger(isImmediate: true)
        case .retryTransfer: swap()
        case .insufficientBalance: break
        case let .useMinimumAmount(value): setFromValue(minimum: value)
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
