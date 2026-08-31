// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemContactService
import GemstonePrimitives
@testable import GemstoneServices
import Primitives
import Store
import StoreTestKit
import Testing

struct ContactServiceTests {
    @Test
    func renamingAContactRenamesItsAddressName() async throws {
        let db = DB.mockWithChains([.ethereum])
        let addressStore = AddressStore(db: db)
        let service = GemContactService(
            store: GemstoneContactStore(store: ContactStore(db: db)),
            addressStore: GemstoneAddressStore(store: addressStore),
            files: GemstoneFileStore(),
        )
        let contactId = UUID().uuidString
        let address = "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326"
        let addresses = try service.addAddress(
            [],
            contactId: contactId,
            chain: .ethereum,
            address: address,
            memo: nil,
            replacingId: nil,
        )

        let created = try await service.saveContact(
            id: contactId,
            existing: nil,
            name: "Alice",
            description: "",
            avatar: .empty,
            addresses: addresses,
        )
        #expect(try addressStore.getAddressName(chain: .ethereum, address: address)?.name == "Alice")

        _ = try await service.saveContact(
            id: contactId,
            existing: created,
            name: "Bob",
            description: "",
            avatar: .empty,
            addresses: addresses,
        )

        let renamed = try #require(try addressStore.getAddressName(chain: .ethereum, address: address))
        #expect(renamed.name == "Bob")
        #expect(renamed.type == .contact)
    }
}
