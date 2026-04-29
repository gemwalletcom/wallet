import AvatarService
import ExplorerService
import Formatters
import Foundation
@testable import NFT
import NFTServiceTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
import WalletServiceTestKit

@MainActor
struct CollectibleViewModelTests {
    @Test
    func tokenIdValue() {
        #expect(CollectibleViewModel.mock(assetData: .mock(asset: .mock(tokenId: "12345"))).tokenIdValue == "12345")
    }

    @Test
    func tokenIdField() {
        let shortModel = CollectibleViewModel.mock(assetData: .mock(asset: .mock(tokenId: "123")))
        let longModel = CollectibleViewModel.mock(assetData: .mock(asset: .mock(tokenId: "1234567890123456789", chain: .ethereum)))

        #expect(shortModel.tokenIdField.value.text == "#123")
        #expect(longModel.tokenIdField.value.text == "1234567...56789")
    }

    @Test
    func contractField() {
        #expect(CollectibleViewModel.mock(assetData: .mock(
            collection: .mock(contractAddress: "0x123"),
            asset: .mock(tokenId: "456"),
        )).contractField?.value.text == "0x123")
        #expect(CollectibleViewModel.mock(assetData: .mock(
            collection: .mock(contractAddress: "0x12345678910"),
            asset: .mock(tokenId: ""),
        )).contractField?.value.text == "0x1234...78910")
        #expect(CollectibleViewModel.mock(assetData: .mock(
            collection: .mock(contractAddress: ""),
            asset: .mock(tokenId: "456"),
        )).contractField == nil)
    }

    @Test
    func tokenExplorerLink() {
        let model = CollectibleViewModel.mock(assetData: .mock(
            collection: .mock(contractAddress: "0x47A00fC8590C11bE4c419D9Ae50DEc267B6E24ee"),
            asset: .mock(tokenId: "11871", chain: .ethereum),
        ))

        #expect(model.tokenIdExplorerLink?.link == "https://etherscan.io/nft/0x47A00fC8590C11bE4c419D9Ae50DEc267B6E24ee/11871")
    }

    @Test
    func showAttributes() {
        #expect(CollectibleViewModel.mock(assetData: .mock(asset: .mock(attributes: []))).showAttributes == false)

        let withAttributesModel = CollectibleViewModel.mock(assetData: .mock(asset: .mock(attributes: [
            NFTAttribute(name: "Color", value: "Blue", percentage: nil),
        ])))
        #expect(withAttributesModel.showAttributes == true)
    }

    @Test
    func attributeValueFormatting() throws {
        let formatter = try RelativeDateFormatter(
            locale: Locale(identifier: "en_US_POSIX"),
            timeZone: #require(TimeZone(secondsFromGMT: 0)),
        )

        let date = NFTAttributeViewModel(
            attribute: NFTAttribute(name: "Created Date", value: "1662714817", valueType: .timestamp, percentage: nil),
            relativeDateFormatter: formatter,
        )
        let string = NFTAttributeViewModel(
            attribute: NFTAttribute(name: "Length", value: "9", valueType: .string, percentage: nil),
            relativeDateFormatter: formatter,
        )

        #expect(date.value == formatter.string(fromTimestampValue: "1662714817"))
        #expect(string.value == "9")
    }

    @Test
    func showLinks() {
        #expect(CollectibleViewModel.mock(assetData: .mock(collection: .mock(links: []))).showLinks == false)
        #expect(CollectibleViewModel.mock(assetData: .mock(collection: .mock(links: [
            AssetLink(name: "Website", url: "https://example.com"),
        ]))).showLinks == true)
    }

    @Test
    func isSendEnabled() {
        let enabledModel = CollectibleViewModel.mock(
            wallet: .mock(type: .multicoin),
            assetData: .mock(asset: .mock(chain: .ethereum)),
        )
        #expect(enabledModel.isSendEnabled == true)

        let viewOnlyModel = CollectibleViewModel.mock(
            wallet: .mock(type: .view),
            assetData: .mock(asset: .mock(chain: .ethereum)),
        )
        #expect(viewOnlyModel.isSendEnabled == false)

        let bitcoinModel = CollectibleViewModel.mock(
            wallet: .mock(type: .multicoin),
            assetData: .mock(asset: .mock(chain: .bitcoin)),
        )
        #expect(bitcoinModel.isSendEnabled == false)
    }
}

// MARK: - Mock Extensions

extension CollectibleViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        assetData: NFTAssetData = .mock(),
        explorerService: ExplorerService = ExplorerService.standard,
    ) -> CollectibleViewModel {
        CollectibleViewModel(
            wallet: wallet,
            assetData: assetData,
            avatarService: AvatarService(store: WalletStore.mock()),
            nftService: .mock(),
            explorerService: explorerService,
            isPresentingSelectedAssetInput: .constant(.none),
        )
    }
}
