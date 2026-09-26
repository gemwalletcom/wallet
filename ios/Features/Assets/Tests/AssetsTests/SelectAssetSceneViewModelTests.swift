// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import AssetsTestKit
import Components
import enum Gemstone.GemSelectAssetState
import enum Gemstone.GemServiceError
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct SelectAssetSceneViewModelTests {
    @Test
    func recentActivityTypes() {
        let model = SelectAssetSceneViewModel.mock()

        #expect(model.recentModel.query.request.types == RecentActivityType.allCases)
    }

    @Test
    func recentsFollowTheChainFilter() {
        let model = SelectAssetSceneViewModel.mock(selectType: .send(.none), chains: [.bitcoin])

        #expect(model.recentModel.query.request.filters == [.enabled, .hasBalance, .chains([Chain.bitcoin.rawValue])])

        _ = model.onFinishChainsSelection(SelectionResult(items: [.ethereum], isConfirmed: false))

        #expect(model.recentModel.query.request.filters == [.enabled, .hasBalance, .chains([Chain.ethereum.rawValue])])
    }

    @Test
    func showEmpty() {
        #expect(listState(SelectAssetSceneViewModel.mock(assets: [])) != .idle)
        #expect(listState(SelectAssetSceneViewModel.mock(assets: [AssetData.mock(metadata: .mock(isPinned: true))])) == .idle)
        #expect(listState(SelectAssetSceneViewModel.mock(assets: [AssetData.mock(metadata: .mock(isPinned: false))])) == .idle)
    }

    @Test
    func showLoading() {
        let pinnedAsset = AssetData.mock(metadata: .mock(isPinned: true))
        #expect(listState(SelectAssetSceneViewModel.mock(assets: [], state: .loading)) == .loading)
        #expect(listState(SelectAssetSceneViewModel.mock(assets: [pinnedAsset], state: .loading)) != .loading)
    }

    @Test
    func filterAndAddTokenRequireFlowAndWalletSupport() {
        let walletWithTokens = Wallet.mock(accounts: [.mock(chain: .ethereum)])
        let singleChainWallet = Wallet.mock(type: .single, accounts: [.mock(chain: .ethereum)])
        let withChains = GemAssetSelectionServiceMock()
        withChains.filterChainsResult = [Chain.ethereum.rawValue]
        let withoutTokens = GemAssetSelectionServiceMock()
        withoutTokens.filterChainsResult = [Chain.ethereum.rawValue]
        withoutTokens.tokensSupported = false

        #expect(SelectAssetSceneViewModel.mock(wallet: walletWithTokens, selectType: .manage, service: withChains).showAddToken == true)
        #expect(SelectAssetSceneViewModel.mock(wallet: walletWithTokens, selectType: .send(.none), service: withChains).showAddToken == false)
        #expect(SelectAssetSceneViewModel.mock(wallet: walletWithTokens, selectType: .manage, service: withoutTokens).showAddToken == false)
        #expect(SelectAssetSceneViewModel.mock(wallet: walletWithTokens, selectType: .manage).showAddToken == false)

        #expect(SelectAssetSceneViewModel.mock(wallet: walletWithTokens, selectType: .manage, service: withChains).showFilter == true)
        #expect(SelectAssetSceneViewModel.mock(wallet: walletWithTokens, selectType: .deposit, service: withChains).showFilter == false)
        #expect(SelectAssetSceneViewModel.mock(wallet: singleChainWallet, selectType: .manage, service: withChains).showFilter == false)
    }

    @Test
    func selectRecentWithoutAccountDoesNotSelectAsset() {
        let model = SelectAssetSceneViewModel.mock(
            wallet: Wallet.mock(accounts: [.mock(chain: .ethereum)]),
            selectType: .send(.none),
        )

        model.onSelectRecent(.mock(id: .mock(chain: .solana), name: "Solana", symbol: "SOL", decimals: 9))
        #expect(model.route == nil)

        model.onSelectRecent(.mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18))
        #expect(model.route != nil)
    }

    @Test
    func toggleFlowEnablesAssets() async {
        await confirmation { enabledAssets in
            let enabler = GemAssetSelectionServiceMock(onSetAssetsEnabled: { assetIds, enabled in
                #expect(assetIds == [AssetId.mock().identifier])
                #expect(enabled == true)
                enabledAssets()
            })
            await SelectAssetSceneViewModel.mock(selectType: .manage, service: enabler)
                .setAssetEnabled(assetId: .mock(), enabled: true)
        }
    }

    @Test
    func nonToggleFlowNeverEnablesAssets() async {
        await confirmation(expectedCount: 0) { enabledAssets in
            let enabler = GemAssetSelectionServiceMock(onSetAssetsEnabled: { _, _ in enabledAssets() })
            await SelectAssetSceneViewModel.mock(selectType: .receive(.asset), service: enabler)
                .setAssetEnabled(assetId: .mock(), enabled: true)
        }
    }

    @Test
    func aFailedToggleShowsTheError() async {
        let model = SelectAssetSceneViewModel.mock(selectType: .manage, service: GemAssetSelectionServiceMock(error: GemServiceError.Offline))

        await model.setAssetEnabled(assetId: .mock(), enabled: true)

        #expect(model.isPresentingToastMessage != nil)
    }

    private func listState(_ model: SelectAssetSceneViewModel) -> GemSelectAssetState {
        model.listState(model.sections)
    }
}
