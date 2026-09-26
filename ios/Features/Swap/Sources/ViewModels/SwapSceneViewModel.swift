// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import class Gemstone.Config
import func Gemstone.formattedPercentage
import struct Gemstone.GemAssetBalance
import enum Gemstone.GemSlippageSelection
import struct Gemstone.GemSwapAssetData
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
    @ObservationIgnored private var viewStateCache: (session: GemSwapSession, pay: GemSwapAssetData?, receive: GemSwapAssetData?, state: GemSwapViewState)?
    public var isPresentingInfoSheet: SwapSheetType?

    public let fromAssetQuery: ObservableQuery<AssetQueryOptional>
    public let toAssetQuery: ObservableQuery<AssetQueryOptional>

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
        let pay = fromAsset?.swapAssetData
        let receive = toAsset?.swapAssetData
        if let cache = viewStateCache, cache.session == session, cache.pay == pay, cache.receive == receive {
            return cache.state
        }
        let state = session.viewState(pay: pay, receive: receive, currency: service.currency.toGem())
        viewStateCache = (session, pay, receive, state)
        return state
    }

    var selectedSwapQuote: SwapperQuote? {
        viewState.quote
    }

    var amountInputModel = InputValidationViewModel()
    var toValue: String = ""
    var loadTrigger: SwapLoadTrigger?

    var selectedSlippage: GemSlippageSelection = .auto

    private let onSwap: TransferDataAction
    private let service: any GemSwapQuoteServiceProtocol

    public init(
        service: any GemSwapQuoteServiceProtocol,
        input: SwapInput,
        onSwap: TransferDataAction = nil,
    ) {
        let pairSelectorModel = input.pairSelector
        self.pairSelectorModel = pairSelectorModel
        self.service = service
        wallet = input.wallet

        fromAssetQuery = ObservableQuery(AssetQueryOptional(walletId: input.wallet.id, assetId: pairSelectorModel.fromAssetId), initialValue: nil)
        toAssetQuery = ObservableQuery(AssetQueryOptional(walletId: input.wallet.id, assetId: pairSelectorModel.toAssetId), initialValue: nil)
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
        let state = viewState
        guard let details = state.details else { return nil }
        return SwapDetailsViewModel(
            state: providersState(state),
            details: details,
            allowSelectProvider: state.allowsProviderSelection,
            swapProviderSelectAction: { [weak self] provider in
                self?.onFinishSwapProviderSelection(provider)
            },
        )
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
            self?.isPresentingInfoSheet = .info(topic.infoSheet)
        }
    }

    func swapTokenModel(type: SelectAssetSwapType) -> SwapTokenViewModel {
        let state = viewState
        return switch type {
        case .pay: SwapTokenViewModel(asset: fromAsset?.asset, side: state.pay)
        case .receive: SwapTokenViewModel(asset: toAsset?.asset, side: state.receive)
        }
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
        guard !isTransferDataLoading, newQuote != nil else { return }
        setToValue()
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
    private func providersState(_ state: GemSwapViewState) -> StateViewType<[SwapProviderItem]> {
        switch state.quotesState {
        case .loading: .loading
        case let .failed(error): .error(error)
        case .quotes: .data(state.providers.map(SwapProviderItem.init(row:)))
        case .empty: .noData
        }
    }

    private func updateSessionInput(amount: String? = nil) {
        session = session.onInputChanged(
            amount: amount ?? amountInputModel.text,
            payAsset: fromAsset?.asset.toGem(),
            receiveAsset: toAsset?.asset.toGem(),
            availableValue: fromAsset?.balance.available ?? .zero,
            slippageBps: selectedSlippage.bps,
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

    private func setToValue() {
        toValue = viewState.receiveAmount?.text() ?? ""
    }

    private func setFromValue(percent: Int, assetData: AssetData) {
        let value = service.amountForPercent(available: assetData.balance.available, percent: UInt32(percent))
        guard let text = NumberInput.format().inputText(value: value.description, decimals: UInt32(assetData.asset.decimals)) else { return }
        amountInputModel.text = text
    }

    private func setMinimumAmount() {
        guard let fromAsset, let text = session.minimumAmountText(payAsset: fromAsset.asset.toGem(), format: NumberInput.format()) else { return }
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
            setToValue()
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
        case .useMinimumAmount: setMinimumAmount()
        case .swap:
            if let warningText = swapDetailsViewModel?.highImpactWarningDescription {
                isPresentingPriceImpactConfirmation = warningText
                return
            }
            swap()
        }
    }
}

private extension AssetData {
    var swapAssetData: GemSwapAssetData {
        GemSwapAssetData(
            asset: asset.toGem(),
            balance: GemAssetBalance(balance, assetId: asset.id, isActive: metadata.isActive),
            price: price?.price,
        )
    }
}
