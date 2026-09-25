import Foundation
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import ImageGalleryServiceTestKit
import Localization
@testable import NFT
import NFTTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

@MainActor
struct CollectibleViewModelTests {
    @Test
    func aSavedImageShowsTheSuccessToastOnlyAfterPhotosFinishes() async {
        let model = CollectibleViewModel.mock(assetData: .mock(asset: .mock(images: .mock(preview: .mock(url: "https://example.com/nft.png")))), gallery: ImageGallerySaverMock(result: {
            try? await Task.sleep(for: .milliseconds(50))
            return nil
        }))

        await model.saveToGallery()

        #expect(model.isPresentingToast != nil)
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func aFailedPhotosWriteShowsTheErrorInsteadOfASuccessToast() async {
        let model = CollectibleViewModel.mock(assetData: .mock(asset: .mock(images: .mock(preview: .mock(url: "https://example.com/nft.png")))), gallery: ImageGallerySaverMock(result: { .saveFailed(AnyError("disk")) }))

        await model.saveToGallery()

        #expect(model.isPresentingToast == nil)
        #expect(model.isPresentingAlertMessage?.message == Localized.Errors.errorOccurred)
    }

    @Test
    func deniedPhotosAccessOffersTheSettings() async {
        let model = CollectibleViewModel.mock(assetData: .mock(asset: .mock(images: .mock(preview: .mock(url: "https://example.com/nft.png")))), gallery: ImageGallerySaverMock(result: { .permissionDenied }))

        await model.saveToGallery()

        #expect(model.isPresentingToast == nil)
        #expect(model.isPresentingAlertMessage?.title == Localized.Permissions.accessDenied)
    }

    @Test
    func canSendOnlyWhileTheWalletHoldsTheAsset() {
        let assetData = NFTAssetData.mock(asset: .mock(chain: .ethereum))
        let held = CollectibleViewModel.mock(assetData: assetData)
        let viewOnly = CollectibleViewModel.mock(wallet: .mock(type: .view), assetData: assetData)

        #expect(sendEnabled(held) == false)
        held.query.value = NFTAssetDetails(assetData: assetData, isOwned: true)
        #expect(sendEnabled(held))
        held.query.value = NFTAssetDetails(assetData: assetData, isOwned: false)
        #expect(sendEnabled(held) == false)

        viewOnly.query.value = NFTAssetDetails(assetData: assetData, isOwned: true)
        #expect(sendEnabled(viewOnly) == false)
    }

    @Test
    func sectionsAndExplorerLinksComeFromCore() throws {
        let model = CollectibleViewModel.mock(assetData: .mock(
            collection: .mock(
                contractAddress: "0x47A00fC8590C11bE4c419D9Ae50DEc267B6E24ee",
                status: .unverified,
                links: [AssetLink(name: "website", url: "https://example.com")],
            ),
            asset: .mock(tokenId: "11871", chain: .ethereum, attributes: [NFTAttribute(name: "Color", value: "Blue", percentage: nil)]),
        ))
        let sections = model.details.sections

        #expect(sections.count == 4)
        #expect(sections.map(\.title) == [.none, .none, .properties, .socialLinks])
        #expect(model.details.isVerified == false)
        guard case let .info(rows) = sections[1].section, case let .identifier(title, copy, explorer) = try #require(rows.last) else {
            Issue.record("expected the token id row to close the info section")
            return
        }
        #expect(title == .tokenId)
        #expect(copy.display == "#11871")
        #expect(explorer?.link == "https://etherscan.io/nft/0x47A00fC8590C11bE4c419D9Ae50DEc267B6E24ee/11871")
    }

    @Test
    func verifiedAssetWithoutExtrasOnlyListsItsInfo() {
        let model = CollectibleViewModel.mock(assetData: .mock(collection: .mock(status: .verified, links: []), asset: .mock(attributes: [])))

        #expect(model.details.sections.count == 1)
        #expect(model.details.isVerified)
    }

    @Test
    func theContractRowOpensAddressDetailsOnTheCollectionChain() {
        var selected: ChainAddress?
        let assetData = NFTAssetData.mock()
        let model = CollectibleViewModel.mock(assetData: assetData, onSelectAddress: { selected = $0 })

        model.onSelectContract?("0xcontract")

        #expect(selected == ChainAddress(chain: assetData.asset.chain, address: "0xcontract"))
        #expect(CollectibleViewModel.mock().onSelectContract == nil)
    }

    private func sendEnabled(_ model: CollectibleViewModel) -> Bool {
        model.headerButtons(model.details).first { $0.type == .send }?.isEnabled == true
    }
}
