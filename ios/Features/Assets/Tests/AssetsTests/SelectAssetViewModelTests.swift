// Copyright (c). Gem Wallet. All rights reserved.

@testable import Assets
import AssetsTestKit
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
        let walletWithoutTokens = Wallet.mock(accounts: [.mock(chain: .bitcoin)])
        let singleChainWallet = Wallet.mock(type: .single, accounts: [.mock(chain: .ethereum)])

        #expect(SelectAssetViewModel.mock(wallet: walletWithTokens, selectType: .manage).showAddToken == true)
        #expect(SelectAssetViewModel.mock(wallet: walletWithTokens, selectType: .send(.none)).showAddToken == false)
        #expect(SelectAssetViewModel.mock(wallet: walletWithoutTokens, selectType: .manage).showAddToken == false)

        #expect(SelectAssetViewModel.mock(wallet: walletWithTokens, selectType: .manage).showFilter == true)
        #expect(SelectAssetViewModel.mock(wallet: walletWithTokens, selectType: .deposit).showFilter == false)
        #expect(SelectAssetViewModel.mock(wallet: singleChainWallet, selectType: .manage).showFilter == false)
    }

    @Test
    func toggleFlowEnablesAssets() async {
        await confirmation { enabledAssets in
            let enabler: AssetsEnablerMock = .mock(onEnableAssets: { _, assetIds, enabled in
                #expect(assetIds == [.mock()])
                #expect(enabled == true)
                enabledAssets()
            })
            await SelectAssetViewModel.mock(selectType: .manage, assetsEnabler: enabler)
                .handleAction(assetId: .mock(), enabled: true)
        }
    }

    @Test
    func nonToggleFlowNeverEnablesAssets() async {
        await confirmation(expectedCount: 0) { enabledAssets in
            let enabler: AssetsEnablerMock = .mock(onEnableAssets: { _, _, _ in enabledAssets() })
            await SelectAssetViewModel.mock(selectType: .receive(.asset), assetsEnabler: enabler)
                .handleAction(assetId: .mock(), enabled: true)
        }
    }
}
