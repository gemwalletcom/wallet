import Foundation
import class Gemstone.GemNftService
import GemstoneServicesTestKit
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
            .mock(collection: .mock(id: .mock(), status: .verified)),
            .mock(collection: .mock(id: NFTCollectionId(chain: .ethereum, contractAddress: "0xunverified"), status: .unverified)),
        ]

        #expect(model.content.unverifiedCount == "1")
    }

    @Test
    func unverifiedCollectionsListsOnlyWhatTheQueryHolds() {
        let model = UnverifiedCollectionsViewModel(service: GemNftService.mock(), wallet: .mock())

        #expect(model.content.isEmpty)

        model.query.value = [.mock(collection: .mock(status: .unverified))]

        #expect(model.content.items.isEmpty == false)
        #expect(model.content.unverifiedCount == nil)
    }

    @Test
    func collectionTitleComesFromTheCollectionItHolds() {
        let model = CollectionViewModel(service: GemNftService.mock(), wallet: .mock(), collectionId: "collection")

        #expect(model.title.isEmpty)

        model.query.value = [.mock(collection: .mock(name: "Punks"))]

        #expect(model.title == "Punks")
    }
}
