import Foundation
import GemstoneServicesTestKit
import Localization
@testable import NFT
import NFTTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing

@MainActor
struct CollectionsViewModelTests {
    @Test
    func unverifiedCountShowsOnlyWhenACollectionIsUnverified() {
        let model = CollectionsViewModel.mock()

        #expect(model.content.unverifiedCount == nil)

        model.query.value = [
            .mock(collection: .mock(id: .mock(), status: .verified), assets: [.mock()]),
            .mock(collection: .mock(id: NFTCollectionId(chain: .ethereum, contractAddress: "0xunverified"), status: .unverified), assets: [.mock()]),
        ]

        #expect(model.content.unverifiedCount == "1")
    }

    @Test
    func aFailedRefreshOverOnlyUnverifiedCollectionsShowsNoErrorRow() {
        let model = CollectionsViewModel.mock()
        model.query.value = [.mock(collection: .mock(status: .unverified), assets: [.mock()])]
        model.loadState = .error(error: .Gateway(msg: "offline"))

        #expect(model.content.items.isEmpty)
        #expect(model.loadError(model.screen) == nil)
    }

    @Test
    func unverifiedCollectionsListsOnlyWhatTheQueryHolds() {
        let model = CollectionsViewModel.mock(list: .unverified)

        #expect(model.screen.hasContent == false)

        model.query.value = [.mock(collection: .mock(status: .unverified), assets: [.mock()])]

        #expect(model.content.items.isEmpty == false)
        #expect(model.content.unverifiedCount == nil)
    }

    @Test
    func collectionTitleComesFromTheCollectionItHolds() {
        let model = CollectionsViewModel.mock(list: .collection, collectionId: "collection")

        #expect(model.screen.title.text == Localized.Nft.collections, "a collection with nothing in it still names the screen")

        model.query.value = [.mock(collection: .mock(name: "Punks"), assets: [.mock()])]

        #expect(model.screen.title.text == "Punks")
        #expect(model.screen.offersReceive)
    }

    @Test
    func nothingUnverifiedIsWorthAskingFor() {
        let model = CollectionsViewModel.mock(list: .unverified)

        #expect(model.screen.title.text == Localized.Asset.Verification.unverified)
        #expect(model.screen.offersReceive == false)
    }
}
