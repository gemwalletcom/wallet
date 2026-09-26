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
struct CollectionsSceneViewModelTests {
    @Test
    func unverifiedCountShowsOnlyWhenACollectionIsUnverified() {
        let model = CollectionsSceneViewModel.mock()

        #expect(model.screen.unverifiedRow == nil)

        model.query.value = [
            .mock(collection: .mock(id: .mock(), status: .verified), assets: [.mock()]),
            .mock(collection: .mock(id: NFTCollectionId(chain: .ethereum, contractAddress: "0xunverified"), status: .unverified), assets: [.mock()]),
        ]

        #expect(model.screen.unverifiedRow?.countText == "1")
    }

    @Test
    func aFailedRefreshOverOnlyUnverifiedCollectionsShowsNoErrorRow() {
        let model = CollectionsSceneViewModel.mock()
        model.query.value = [.mock(collection: .mock(status: .unverified), assets: [.mock()])]
        model.loadState = .error(error: .Gateway(msg: "offline"))

        #expect(model.screen.items.isEmpty)
        #expect(model.loadError(model.screen) == nil)
    }

    @Test
    func unverifiedCollectionsListsOnlyWhatTheQueryHolds() {
        let model = CollectionsSceneViewModel.mock(list: .unverified)

        #expect(model.screen.hasContent == false)

        model.query.value = [.mock(collection: .mock(status: .unverified), assets: [.mock()])]

        #expect(model.screen.items.isEmpty == false)
        #expect(model.screen.unverifiedRow == nil)
    }

    @Test
    func collectionTitleComesFromTheCollectionItHolds() {
        let model = CollectionsSceneViewModel.mock(list: .collection, collectionId: "collection")

        #expect(model.screen.title.text == Localized.Nft.collections, "a collection with nothing in it still names the screen")

        model.query.value = [.mock(collection: .mock(name: "Punks"), assets: [.mock()])]

        #expect(model.screen.title.text == "Punks")
        #expect(model.screen.emptyState.actions == [.receive])
    }

    @Test
    func nothingUnverifiedIsWorthAskingFor() {
        let model = CollectionsSceneViewModel.mock(list: .unverified)

        #expect(model.screen.title.text == Localized.Asset.Verification.unverified)
        #expect(model.screen.emptyState.actions.isEmpty)
    }
}
