// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemSwapPairSuggestion
import struct Gemstone.SwapperQuote
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Observation
import Primitives
import PrimitivesTestKit
@testable import Store
@testable import Swap
import SwapTestKit
import Testing

@MainActor
struct SwapSceneViewModelTests {
    @Test
    func suggestPairAppliesTheCoreSuggestion() async {
        let suggestion = GemSwapPairSuggestion(payAssetId: AssetId.mock(chain: .ethereum).identifier, receiveAssetId: AssetId.mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7").identifier)
        let model = SwapSceneViewModel.mock(service: GemSwapQuoteServiceMock(pairSuggestion: suggestion))

        await model.suggestPair()

        #expect(model.pairSelectorModel.fromAssetId == .mock(chain: .ethereum))
        #expect(model.pairSelectorModel.toAssetId == .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"))
    }

    @Test
    func suggestPairKeepsAnAlreadySelectedReceiveAsset() async {
        let suggestion = GemSwapPairSuggestion(payAssetId: AssetId.mock(chain: .ethereum).identifier, receiveAssetId: AssetId.mock(chain: .solana).identifier)
        let model = SwapSceneViewModel.mock(
            service: GemSwapQuoteServiceMock(pairSuggestion: suggestion),
            pairSelector: SwapPairSelectorViewModel(fromAssetId: .mock(chain: .ethereum), toAssetId: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7")),
        )

        await model.suggestPair()

        #expect(model.pairSelectorModel.toAssetId == .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"))
    }

    @Test
    func toValue() async {
        let cases: [(BigUInt, String)] = [(250_000_000_000, "250,000"), (1_000_000, "1"), (10000, "0.01"), (12, "0.000012")]
        for (toValue, expected) in cases {
            let model = SwapSceneViewModel.mock(service: GemSwapQuoteServiceMock(quotes: [.mock(toValue: toValue, request: .mock(toAsset: .mock(decimals: 6)))]))
            await model.load()
            #expect(model.toValue == expected)
        }
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

    @Test(.timeLimit(.minutes(1)))
    func refreshKeepsReceiveValueUntilTheNewQuoteArrives() async {
        let (answers, answer) = AsyncStream<[SwapperQuote]>.makeStream()
        let model = SwapSceneViewModel.mock(service: GemSwapQuoteServiceMock(quotes: { _ in await answers.first { _ in true } ?? [] }))
        answer.yield([.mock(toValue: 1_000_000, request: .mock(toAsset: .mock(decimals: 6)))])
        await model.load()

        let refresh = Task { await model.load() }
        while !model.isReceiveFieldLoading {
            await withCheckedContinuation { changed in
                withObservationTracking { _ = model.session } onChange: { changed.resume() }
            }
        }

        #expect(model.toValue == "1")

        answer.yield([.mock(toValue: 2_000_000, request: .mock(toAsset: .mock(decimals: 6)))])
        await refresh.value

        #expect(model.toValue == "2")
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
    func changingReceiveAssetPreservesInputAmount() async throws {
        let model = SwapSceneViewModel.mock()
        await model.load()
        let oldAsset = model.toAsset

        let transfer = try #require(model.session.startTransfer())
        model.session = transfer.onTransferFailed(transfer: transfer.transferPhase, error: .TransactionError("nonce"))
        model.loadTrigger = nil
        model.toAssetQuery.value = .mock(asset: .mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18))
        model.onChangeToAsset(old: oldAsset, new: model.toAsset)

        #expect(model.amountInputModel.text == "1")
        #expect(model.toValue.isEmpty)
        #expect(model.selectedSwapQuote == nil)
        #expect(model.viewState.error == nil)
        #expect(model.loadTrigger?.isImmediate == true)
    }

    @Test
    func changingPayAssetClearsInputAmount() async {
        let model = SwapSceneViewModel.mock()
        await model.load()
        let oldAsset = model.fromAsset

        model.fromAssetQuery.value = .mock(asset: .mock(id: .mock(chain: .smartChain), name: "BNB", symbol: "BNB", decimals: 18), balance: .mock())
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
        model.onChangeToAsset(
            old: .mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)),
            new: .mock(asset: .mock(id: .mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7"), name: "Tether", symbol: "USDT", decimals: 6, type: .erc20)),
        )

        #expect(model.loadTrigger?.isImmediate == true)

        model.loadTrigger = nil
        model.session = .mock(quotePhase: .failed(request: .mock(), error: .NoQuoteAvailable), input: .mock())
        model.buttonViewModel.action()

        #expect(model.loadTrigger?.isImmediate == true)

        model.loadTrigger = nil
        model.session = .mock(quotePhase: .failed(request: .mock(), error: .InputAmountError(minAmount: "1000000000000000000")), input: .mock())
        model.buttonViewModel.action()

        #expect(model.loadTrigger?.isImmediate == true)
    }

    @Test
    func retryQuoteUpdatesLoadTrigger() {
        let model = SwapSceneViewModel.mock()

        model.session = .mock(quotePhase: .failed(request: .mock(), error: .NoQuoteAvailable), input: .mock())
        model.buttonViewModel.action()
        let firstRetry = model.loadTrigger
        #expect(model.viewState.isQuoteLoading)

        model.session = .mock(quotePhase: .failed(request: .mock(), error: .NoQuoteAvailable), input: .mock())
        model.buttonViewModel.action()

        #expect(model.loadTrigger != firstRetry)
    }

    @Test
    func providerSelectionAppliesWithoutRefetch() async {
        let service = GemSwapQuoteServiceMock(
            quotes: [
                .mock(toValue: 260_000_000_000, data: .mock(provider: .mock(id: .uniswapV3))),
                .mock(toValue: 250_000_000_000, data: .mock(provider: .mock(id: .thorchain))),
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

        model.session = model.session.onFetchStarted(request: .mock())

        #expect(model.viewState.isQuoteLoading)
        #expect(model.selectedSwapQuote != nil)
        #expect(model.swapDetails != nil)
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
        #expect(Set(service.priceSubscriptions[0]) == Set([AssetId.mock(chain: .ethereum), AssetId.mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7")].map(\.identifier)))

        model.amountInputModel.text = "2"
        model.onChangeFromValue("1", "2")
        model.onSelectPercent(100)
        model.onSelectSlippage(.manual(bps: 150))
        #expect(service.priceSubscriptions.count == 1, "typing, Max and slippage do not touch the pair")

        model.toAssetQuery.value = .mock(asset: .mock(id: .mock(chain: .solana), name: "Solana", symbol: "SOL", decimals: 9))
        await model.onAssetIdsChange(assetIds: model.assetIds)
        #expect(service.priceSubscriptions.count == 2)
        #expect(service.balanceUpdates.count == 2)
    }
}
