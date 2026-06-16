// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import AssetsServiceTestKit
import BlockchainTestKit
import ChainServiceTestKit
import GemAPITestKit
import Primitives
import PrimitivesTestKit
import Testing

struct AssetsServiceTests {
    private let tokenId = "0x227D920e20eBAc8A40E7D6431B7d724Bb64D7245"

    @Test
    func searchMergesBackendAndNodeResults() async throws {
        let apiAsset = AssetBasic.mock(asset: .mock(id: AssetId(chain: .ethereum, tokenId: "0xapi")))
        let nodeAsset = Asset.mock(id: AssetId(chain: .smartChain, tokenId: tokenId))

        let result = try await search(apiAssets: [apiAsset], nodeAsset: nodeAsset, chains: [.smartChain])

        #expect(result.map(\.asset.id) == [apiAsset.asset.id, nodeAsset.id])
    }

    @Test
    func searchResolvesViaNodeWhenBackendEmpty() async throws {
        let nodeAsset = Asset.mock(id: AssetId(chain: .ethereum, tokenId: tokenId))

        let result = try await search(apiAssets: [], nodeAsset: nodeAsset, chains: [.ethereum])

        #expect(result.map(\.asset.id) == [nodeAsset.id])
    }

    private func search(apiAssets: [AssetBasic], nodeAsset: Asset, chains: [Chain]) async throws -> [AssetBasic] {
        let chainService = ChainServiceMock()
        chainService.tokenData[tokenId] = nodeAsset
        let service = AssetsService.mock(
            chainServiceFactory: ChainServiceFactoryMock(chainService: chainService),
            assetsProvider: GemAPIAssetsServiceMock(searchAssetsResult: apiAssets),
        )
        return try await service.searchAssets(query: tokenId, chains: chains, tags: [])
    }
}
