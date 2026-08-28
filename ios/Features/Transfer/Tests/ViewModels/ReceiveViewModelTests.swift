// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer

struct ReceiveViewModelTests {
    @MainActor
    @Test
    func associationsIncludeSupportedNetworksOnce() {
        let tronAsset = Asset.mockTronUSDT()
        let ethereumAsset = Asset.mockEthereumUSDT()
        let solanaAssetId = AssetId(chain: .solana, tokenId: "usdt")
        let wallet = Wallet.mock(
            accounts: [
                .mock(chain: .tron, address: "tron"),
                .mock(chain: .ethereum, address: "ethereum"),
            ],
        )
        let model = ReceiveViewModel(
            assetData: .mock(
                asset: tronAsset,
                account: .mock(chain: .tron, address: "tron"),
                associations: [
                    AssetAssociation(assetId: ethereumAsset.id, type: .official),
                    AssetAssociation(assetId: ethereumAsset.id, type: .official),
                    AssetAssociation(assetId: solanaAssetId, type: .official),
                ],
            ),
            wallet: wallet,
            balanceService: .mock(),
            assetsService: GemAssetsServiceMock(),
        )

        #expect(model.networkAssetIds == [tronAsset.id, ethereumAsset.id])
        #expect(model.showNetworkSelector)
    }
}
