import Foundation
import class Gemstone.GemCollectibleService
import class Gemstone.GemExplorerService
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
@testable import NFT
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

@MainActor
struct CollectibleViewModelTests {
    @Test
    func canSendOnlyWhileTheWalletHoldsTheAsset() {
        let assetData = NFTAssetData.mock(asset: .mock(chain: .ethereum))
        let held = CollectibleViewModel.mock(assetData: assetData)
        let viewOnly = CollectibleViewModel.mock(wallet: .mock(type: .view), assetData: assetData)

        #expect(held.details.canSend == false)
        held.query.value = NFTAssetDetails(assetData: assetData, isOwned: true)
        #expect(held.details.canSend)
        held.query.value = NFTAssetDetails(assetData: assetData, isOwned: false)
        #expect(held.details.canSend == false)

        viewOnly.query.value = NFTAssetDetails(assetData: assetData, isOwned: true)
        #expect(viewOnly.details.canSend == false)
    }

    @Test
    func sectionsAndExplorerLinksComeFromCore() throws {
        let model = CollectibleViewModel.mock(assetData: .mock(
            collection: .mock(
                contractAddress: "0x47A00fC8590C11bE4c419D9Ae50DEc267B6E24ee",
                status: .unverified,
                links: [AssetLink(name: "Website", url: "https://example.com")],
            ),
            asset: .mock(tokenId: "11871", chain: .ethereum, attributes: [NFTAttribute(name: "Color", value: "Blue", percentage: nil)]),
        ))
        let sections = model.sections

        #expect(sections.count == 4)
        guard case let .info(rows) = sections[1], case let .tokenId(identifier) = try #require(rows.last) else {
            Issue.record("expected the token id row to close the info section")
            return
        }
        #expect(identifier.text == "#11871")
        #expect(identifier.explorer?.link == "https://etherscan.io/nft/0x47A00fC8590C11bE4c419D9Ae50DEc267B6E24ee/11871")
    }

    @Test
    func verifiedAssetWithoutExtrasOnlyListsItsInfo() {
        let model = CollectibleViewModel.mock(assetData: .mock(collection: .mock(status: .verified, links: []), asset: .mock(attributes: [])))

        #expect(model.sections.count == 1)
    }
}

// MARK: - Mock Extensions

extension CollectibleViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        assetData: NFTAssetData = .mock(),
        explorerService: GemExplorerService = .mock(),
    ) -> CollectibleViewModel {
        CollectibleViewModel(
            wallet: wallet,
            assetData: assetData,
            service: GemCollectibleService.mock(explorer: explorerService),
            isPresentingSelectedAssetInput: .constant(.none),
        )
    }
}
