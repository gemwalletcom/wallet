import Foundation
import class Gemstone.GemNftService
import GemstoneServicesTestKit
@testable import NFT
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing

@MainActor
struct CollectionsViewModelTests {
    @Test
    func unverifiedCountShowsOnlyWhenACollectionIsUnverified() {
        let model = CollectionsViewModel(service: GemNftService.mock(), wallet: .mock())

        #expect(model.content.unverifiedCount == nil)

        model.query.value = [
            NFTData(collection: .mock(id: .mock(), status: .verified), assets: [.mock()]),
            NFTData(collection: .mock(id: NFTCollectionId(chain: .ethereum, contractAddress: "0xunverified"), status: .unverified), assets: [.mock()]),
        ]

        #expect(model.content.unverifiedCount == "1")
    }

    @Test
    func unverifiedCollectionsListsOnlyWhatTheQueryHolds() {
        let model = UnverifiedCollectionsViewModel(service: GemNftService.mock(), wallet: .mock())

        #expect(model.content.isEmpty)

        model.query.value = [NFTData(collection: .mock(status: .unverified), assets: [.mock()])]

        #expect(model.content.items.isEmpty == false)
        #expect(model.content.unverifiedCount == nil)
    }

    @Test
    func collectionTitleComesFromTheCollectionItHolds() {
        let model = CollectionViewModel(service: GemNftService.mock(), wallet: .mock(), collectionId: "collection")

        #expect(model.title.isEmpty)

        model.query.value = [NFTData(collection: .mock(name: "Punks"), assets: [.mock()])]

        #expect(model.title == "Punks")
    }
}
