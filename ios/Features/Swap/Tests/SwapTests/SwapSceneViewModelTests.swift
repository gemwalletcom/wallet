// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GemSwapButtonAction
import struct Gemstone.GemSwapPairSuggestion
import struct Gemstone.GemSwapQuotesResult
import struct Gemstone.GemSwapSession
import enum Gemstone.SwapperError
import struct Gemstone.SwapperQuote
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
@testable import Swap
import SwapTestKit
import Testing

@MainActor
struct SwapSceneViewModelTests {
    @Test
    func assetIdsIgnoresPairOrder() {
        let model = SwapSceneViewModel.mock()
        let assetIds = model.assetIds

        model.fromAssetQuery.value = .mock(asset: .mockEthereumUSDT())
        model.toAssetQuery.value = .mock(asset: .mockEthereum(), balance: .mock())

        #expect(model.assetIds == assetIds)
        #expect(model.assetIds == [AssetId.mockEthereum(), AssetId.mockEthereumUSDT()])
    }

    @Test
    func suggestPairAppliesTheCoreSuggestion() async {
        let suggestion = GemSwapPairSuggestion(payAssetId: AssetId.mockEthereum().identifier, receiveAssetId: AssetId.mockEthereumUSDT().identifier)
        let model = SwapSceneViewModel.mock(service: GemSwapQuoteServiceMock(pairSuggestion: suggestion))

        await model.suggestPair()

        #expect(model.pairSelectorModel.fromAssetId == .mockEthereum())
        #expect(model.pairSelectorModel.toAssetId == .mockEthereumUSDT())
    }

    @Test
    func suggestPairKeepsAnAlreadySelectedReceiveAsset() async {
        let suggestion = GemSwapPairSuggestion(payAssetId: AssetId.mockEthereum().identifier, receiveAssetId: AssetId.mockSolana().identifier)
        let model = SwapSceneViewModel.mock(
            service: GemSwapQuoteServiceMock(pairSuggestion: suggestion),
            pairSelector: SwapPairSelectorViewModel(fromAssetId: .mockEthereum(), toAssetId: .mockEthereumUSDT()),
        )

        await model.suggestPair()

        #expect(model.pairSelectorModel.toAssetId == .mockEthereumUSDT())
    }

    @Test
    func toValue() async {
        let cases: [(BigUInt, String)] = [(250_000_000_000, "250,000"), (1_000_000, "1"), (10000, "0.01"), (12, "0.000012")]
        for (toValue, expected) in cases {
            let model = SwapSceneViewModel.mock(service: GemSwapQuoteServiceMock(quotes: [.mock(toValue: toValue)]))
            await model.load()
            #expect(model.toValue == expected)
        }
    }

    @Test
    func additionalInfoVisibility() {
        let model = SwapSceneViewModel.mock()

        model.session = .mockLoading()
        #expect(model.shouldShowAdditionalInfo == false)

        model.session = .mockReady()
        #expect(model.shouldShowAdditionalInfo)
    }

    @Test
    func buttonViewModelFlow() {
        let model = SwapSceneViewModel.mock()

        model.session = .mockReady()
        #expect(model.buttonViewModel.buttonAction == GemSwapButtonAction.swap)
        #expect(model.buttonViewModel.isVisible)

        model.session = .mockFailed(.NoQuoteAvailable)
        #expect(model.buttonViewModel.buttonAction == GemSwapButtonAction.retryQuote)

        model.session = .mockFailed(.InputAmountError(minAmount: "1000"))
        #expect(model.buttonViewModel.buttonAction == GemSwapButtonAction.useMinimumAmount(value: 1000))

        model.session = .mockReady().failedTransfer(.NoQuoteAvailable)
        #expect(model.buttonViewModel.buttonAction == GemSwapButtonAction.retryTransfer)

        model.session = model.session.onQuoteResults(results: GemSwapQuotesResult(request: .mock, quotes: [], error: .NoQuoteAvailable))
        #expect(model.buttonViewModel.buttonAction == GemSwapButtonAction.retryTransfer)
    }

    @Test
    func loadingFlagsSeparateQuoteAndTransferDataStates() throws {
        let model = SwapSceneViewModel.mock()

        model.session = .mockLoading()
        #expect(model.viewState.isQuoteLoading)
        #expect(model.isTransferDataLoading == false)
        #expect(model.viewState.pay.interaction.isAmountEditable)
        #expect(model.isReceiveFieldLoading)

        model.session = try #require(GemSwapSession.mockReady().startTransfer())
        #expect(model.viewState.isQuoteLoading == false)
        #expect(model.isTransferDataLoading)
        #expect(model.viewState.pay.interaction.isAmountEditable == false)
        #expect(model.isReceiveFieldLoading == false)
    }

    @Test
    func fetchDoesNotRunWhileTransferDataLoading() async throws {
        let model = SwapSceneViewModel.mock()
        await model.load()
        let previousToValue = model.toValue
        let previousQuote = model.selectedSwapQuote

        model.session = try #require(model.session.startTransfer())
        await model.load()

        #expect(model.viewState.isQuoteLoading == false)
        #expect(model.toValue == previousToValue)
        #expect(model.selectedSwapQuote == previousQuote)
    }

    @Test
    func quoteChangingActionsClearTransferStateAndDisableProviderSelection() async throws {
        let model = SwapSceneViewModel.mock()
        await model.load()

        model.session = model.session.failedTransfer(.TransactionError("nonce"))
        #expect(model.session.error() != nil)

        model.onFinishSwapProviderSelection(SwapperQuote.mock().data.provider.id)
        #expect(model.session.error() == nil)

        model.session = try #require(model.session.startTransfer())
        #expect(model.swapDetailsViewModel?.allowSelectProvider == false)
    }

    @Test
    func changingAmountClearsReceiveValueBeforeFetch() async {
        let model = SwapSceneViewModel.mock()
        await model.load()

        #expect(model.toValue.isNotEmpty)

        model.amountInputModel.text = "2"
        model.onChangeFromValue("1", "2")

        #expect(model.isReceiveFieldLoading)
        #expect(model.toValue.isEmpty)
        #expect(model.loadTrigger?.isImmediate == false)

        await model.load()

        #expect(model.isReceiveFieldLoading == false)
        #expect(model.toValue.isNotEmpty)
    }

    @Test
    func changingSlippageClearsReceiveValueBeforeFetch() async {
        let model = SwapSceneViewModel.mock()
        await model.load()

        #expect(model.toValue.isNotEmpty)

        model.onSelectSlippage(.manual(bps: 150))

        #expect(model.isReceiveFieldLoading)
        #expect(model.toValue.isEmpty)
        #expect(model.loadTrigger?.isImmediate == true)
    }

    @Test
    func clearingInputResetsQuoteImmediately() async {
        let model = SwapSceneViewModel.mock()
        await model.load()

        #expect(model.toValue.isNotEmpty)
        #expect(model.selectedSwapQuote != nil)

        model.amountInputModel.text = .empty
        model.onChangeFromValue("1", .empty)

        #expect(model.viewState.isInputEmpty)
        #expect(model.toValue.isEmpty)
        #expect(model.selectedSwapQuote == nil)
    }

    @Test
    func changingReceiveAssetPreservesInputAmount() async {
        let model = SwapSceneViewModel.mock()
        await model.load()
        let oldAsset = model.toAsset

        model.session = model.session.failedTransfer(.TransactionError("nonce"))
        model.loadTrigger = nil
        model.toAssetQuery.value = .mock(asset: .mockBNB())
        model.onChangeToAsset(old: oldAsset, new: model.toAsset)

        #expect(model.amountInputModel.text == "1")
        #expect(model.toValue.isEmpty)
        #expect(model.selectedSwapQuote == nil)
        #expect(model.session.error() == nil)
        #expect(model.loadTrigger?.isImmediate == true)
    }

    @Test
    func changingPayAssetClearsInputAmount() async {
        let model = SwapSceneViewModel.mock()
        await model.load()
        let oldAsset = model.fromAsset

        model.fromAssetQuery.value = .mock(asset: .mockBNB(), balance: .mock())
        model.onChangeFromAsset(old: oldAsset, new: model.fromAsset)

        #expect(model.amountInputModel.text.isEmpty)
        #expect(model.toValue.isEmpty)
        #expect(model.selectedSwapQuote == nil)
    }

    @Test
    func loadTriggerIsImmediate() {
        let model = SwapSceneViewModel.mock()

        model.loadTrigger = nil
        model.onChangeFromValue("1", "2")

        #expect(model.loadTrigger?.isImmediate == false)

        model.loadTrigger = nil
        model.onSelectPercent(50)

        #expect(model.loadTrigger?.isImmediate == true)

        model.loadTrigger = nil
        model.onChangeToAsset(old: .mock(asset: .mockEthereum()), new: .mock(asset: .mockEthereumUSDT()))

        #expect(model.loadTrigger?.isImmediate == true)

        model.loadTrigger = nil
        model.session = .mockFailed(.NoQuoteAvailable)
        model.buttonViewModel.action()

        #expect(model.loadTrigger?.isImmediate == true)

        model.loadTrigger = nil
        model.session = .mockFailed(.InputAmountError(minAmount: "1000000000000000000"))
        model.buttonViewModel.action()

        #expect(model.loadTrigger?.isImmediate == true)
    }

    @Test
    func retryQuoteUpdatesLoadTrigger() {
        let model = SwapSceneViewModel.mock()

        model.session = .mockFailed(.NoQuoteAvailable)
        model.buttonViewModel.action()
        let firstRetry = model.loadTrigger
        #expect(model.viewState.isQuoteLoading)

        model.session = .mockFailed(.NoQuoteAvailable)
        model.buttonViewModel.action()

        #expect(model.loadTrigger != firstRetry)
    }

    @Test
    func refreshedQuotesKeepSelectedProvider() async {
        let service = GemSwapQuoteServiceMock(
            quotes: [
                .mock(toValue: 260_000_000_000, provider: .uniswapV3),
                .mock(toValue: 250_000_000_000, provider: .thorchain),
            ],
        )
        let model = SwapSceneViewModel.mock(service: service)

        model.onFinishSwapProviderSelection(.thorchain)
        await model.load()

        #expect(model.selectedSwapQuote?.data.provider.id == .thorchain)
        #expect(model.selectedSwapQuote?.toValue == 250_000_000_000)
    }

    @Test
    func providerSelectionAppliesWithoutRefetch() async {
        let service = GemSwapQuoteServiceMock(
            quotes: [
                .mock(toValue: 260_000_000_000, provider: .uniswapV3),
                .mock(toValue: 250_000_000_000, provider: .thorchain),
            ],
        )
        let model = SwapSceneViewModel.mock(service: service)
        await model.load()

        #expect(model.selectedSwapQuote?.data.provider.id == .uniswapV3)

        model.onFinishSwapProviderSelection(.thorchain)

        #expect(model.selectedSwapQuote?.data.provider.id == .thorchain)
    }

    @Test
    func selectedQuoteSurvivesQuotesReload() async {
        let model = SwapSceneViewModel.mock()
        await model.load()

        model.session = model.session.onFetchStarted(request: .mock)

        #expect(model.viewState.isQuoteLoading)
        #expect(model.selectedSwapQuote != nil)
        #expect(model.swapDetailsViewModel != nil)
    }

    @Test
    func increasedAmountSelectsBestProviderWithoutManualSelection() async {
        let quotesByAmount: @Sendable (BigInt) -> [SwapperQuote] = { amount in
            guard amount > BigInt(2_000_000_000_000_000_000) else {
                return [.mock(toValue: 250_000_000_000, provider: .thorchain)]
            }
            return [
                .mock(toValue: 260_000_000_000, provider: .uniswapV3),
                .mock(toValue: 250_000_000_000, provider: .thorchain),
            ]
        }
        let model = SwapSceneViewModel.mock(service: GemSwapQuoteServiceMock(quotes: quotesByAmount))

        await model.load()

        #expect(model.selectedSwapQuote?.data.provider.id == .thorchain)

        model.amountInputModel.text = "4"
        model.onChangeFromValue("1", "4")
        await model.load()

        #expect(model.selectedSwapQuote?.data.provider.id == .uniswapV3)
        #expect(model.selectedSwapQuote?.toValue == 260_000_000_000)
    }

    @Test
    func refreshedQuotesFallBackWhenSelectedProviderDisappears() async {
        let service = GemSwapQuoteServiceMock(quotes: [.mock(toValue: 260_000_000_000, provider: .uniswapV3)])
        let model = SwapSceneViewModel.mock(service: service)

        model.onFinishSwapProviderSelection(.thorchain)
        await model.load()

        #expect(model.selectedSwapQuote?.data.provider.id == .uniswapV3)
    }

    @Test
    func changedPairDropsManualProviderSelection() async {
        let service = GemSwapQuoteServiceMock(
            quotes: [
                .mock(toValue: 260_000_000_000, provider: .uniswapV3),
                .mock(toValue: 250_000_000_000, provider: .thorchain),
            ],
        )
        let model = SwapSceneViewModel.mock(service: service)

        model.onFinishSwapProviderSelection(.thorchain)
        model.toAssetQuery.value = .mock(asset: .mockSolana())
        model.onChangeToAsset(old: .mock(asset: .mockEthereumUSDT()), new: .mock(asset: .mockSolana()))
        await model.load()

        #expect(model.selectedSwapQuote?.data.provider.id == .uniswapV3)
    }

    @Test
    func slippagePersistsAcrossSessions() {
        let service = GemSwapQuoteServiceMock()
        let model = SwapSceneViewModel.mock(service: service)
        #expect(model.selectedSlippage == .auto)

        model.onSelectSlippage(.manual(bps: 150))

        #expect(service.slippageBps() == 150)
        #expect(SwapSceneViewModel.mock(service: service).selectedSlippage == .manual(bps: 150))
    }

    @Test
    func thePairRefreshesPricesAndBalancesOnceAndNotForEveryEdit() async {
        let service = GemSwapQuoteServiceMock()
        let model = SwapSceneViewModel.mock(service: service)

        await model.onAssetIdsChange(assetIds: model.assetIds)
        #expect(service.priceSubscriptions.count == 1)
        #expect(service.balanceUpdates.count == 1, "the balance the Max button spends from is refreshed with the prices")
        #expect(Set(service.priceSubscriptions[0]) == Set([AssetId.mockEthereum(), AssetId.mockEthereumUSDT()].map(\.identifier)))

        model.amountInputModel.text = "2"
        model.onChangeFromValue("1", "2")
        model.onSelectPercent(100)
        model.onSelectSlippage(.manual(bps: 150))
        #expect(service.priceSubscriptions.count == 1, "typing, Max and slippage do not touch the pair")

        model.toAssetQuery.value = .mock(asset: .mockSolana())
        await model.onAssetIdsChange(assetIds: model.assetIds)
        #expect(service.priceSubscriptions.count == 2)
        #expect(service.balanceUpdates.count == 2)
    }
}
