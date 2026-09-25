// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import AssetsTestKit
import enum Gemstone.GemSelectAssetState
import enum Gemstone.GemServiceError
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct SelectAssetViewModelTests {
    @Test
    func recentActivityTypes() {
        let model = SelectAssetViewModel.mock()

        #expect(model.recentModel.query.request.types == RecentActivityType.allCases)
    }

    @Test
    func recentsFollowTheChainFilter() {
        let model = SelectAssetViewModel.mock(selectType: .send(.none), chains: [.bitcoin])

        #expect(model.recentModel.query.request.filters == [.enabled, .hasBalance, .chains([Chain.bitcoin.rawValue])])

        var filter = model.filterModel
        filter.chainsFilter.selectedChains = [.ethereum]
        model.onChangeFilterModel(model.filterModel, model: filter)

        #expect(model.recentModel.query.request.filters == [.enabled, .hasBalance, .chains([Chain.ethereum.rawValue])])
    }

    @Test
    func showEmpty() {
        #expect(listState(SelectAssetViewModel.mock(assets: [])) != .idle)
        #expect(listState(SelectAssetViewModel.mock(assets: [AssetData.mock(metadata: .mock(isPinned: true))])) == .idle)
        #expect(listState(SelectAssetViewModel.mock(assets: [AssetData.mock(metadata: .mock(isPinned: false))])) == .idle)
    }

    @Test
    func showLoading() {
        let pinnedAsset = AssetData.mock(metadata: .mock(isPinned: true))
        #expect(listState(SelectAssetViewModel.mock(assets: [], state: .loading)) == .loading)
        #expect(listState(SelectAssetViewModel.mock(assets: [pinnedAsset], state: .loading)) != .loading)
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

        #expect(SelectAssetViewModel.mock(wallet: walletWithTokens, selectType: .manage, service: withChains).showAddToken == true)
        #expect(SelectAssetViewModel.mock(wallet: walletWithTokens, selectType: .send(.none), service: withChains).showAddToken == false)
        #expect(SelectAssetViewModel.mock(wallet: walletWithTokens, selectType: .manage, service: withoutTokens).showAddToken == false)
        #expect(SelectAssetViewModel.mock(wallet: walletWithTokens, selectType: .manage).showAddToken == false)

        #expect(SelectAssetViewModel.mock(wallet: walletWithTokens, selectType: .manage, service: withChains).showFilter == true)
        #expect(SelectAssetViewModel.mock(wallet: walletWithTokens, selectType: .deposit, service: withChains).showFilter == false)
        #expect(SelectAssetViewModel.mock(wallet: singleChainWallet, selectType: .manage, service: withChains).showFilter == false)
    }

    @Test
    func selectRecentWithoutAccountDoesNotSelectAsset() {
        let model = SelectAssetViewModel.mock(
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
            await SelectAssetViewModel.mock(selectType: .manage, service: enabler)
                .setAssetEnabled(assetId: .mock(), enabled: true)
        }
    }

    @Test
    func nonToggleFlowNeverEnablesAssets() async {
        await confirmation(expectedCount: 0) { enabledAssets in
            let enabler = GemAssetSelectionServiceMock(onSetAssetsEnabled: { _, _ in enabledAssets() })
            await SelectAssetViewModel.mock(selectType: .receive(.asset), service: enabler)
                .setAssetEnabled(assetId: .mock(), enabled: true)
        }
    }

    @Test
    func aFailedToggleShowsTheError() async {
        let model = SelectAssetViewModel.mock(selectType: .manage, service: GemAssetSelectionServiceMock(error: GemServiceError.Offline))

        await model.setAssetEnabled(assetId: .mock(), enabled: true)

        #expect(model.isPresentingToastMessage != nil)
    }

    private func listState(_ model: SelectAssetViewModel) -> GemSelectAssetState {
        model.listState(model.sections)
    }
}
