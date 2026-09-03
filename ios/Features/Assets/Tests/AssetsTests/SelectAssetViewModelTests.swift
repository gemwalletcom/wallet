// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import AssetsTestKit
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
    func showEmpty() {
        #expect(SelectAssetViewModel.mock(assets: []).showEmpty == true)
        #expect(SelectAssetViewModel.mock(assets: [AssetData.mock(metadata: .mock(isPinned: true))]).showEmpty == false)
        #expect(SelectAssetViewModel.mock(assets: [AssetData.mock(metadata: .mock(isPinned: false))]).showEmpty == false)
    }

    @Test
    func showLoading() {
        let pinnedAsset = AssetData.mock(metadata: .mock(isPinned: true))
        #expect(SelectAssetViewModel.mock(assets: [], state: .loading).showLoading == true)
        #expect(SelectAssetViewModel.mock(assets: [pinnedAsset], state: .loading).showLoading == false)
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
    func toggleFlowEnablesAssets() async {
        await confirmation { enabledAssets in
            let enabler = GemAssetSelectionServiceMock(onSetAssetsEnabled: { assetIds, enabled in
                #expect(assetIds == [AssetId.mock().identifier])
                #expect(enabled == true)
                enabledAssets()
            })
            await SelectAssetViewModel.mock(selectType: .manage, service: enabler)
                .handleAction(assetId: .mock(), enabled: true)
        }
    }

    @Test
    func nonToggleFlowNeverEnablesAssets() async {
        await confirmation(expectedCount: 0) { enabledAssets in
            let enabler = GemAssetSelectionServiceMock(onSetAssetsEnabled: { _, _ in enabledAssets() })
            await SelectAssetViewModel.mock(selectType: .receive(.asset), service: enabler)
                .handleAction(assetId: .mock(), enabled: true)
        }
    }
}
