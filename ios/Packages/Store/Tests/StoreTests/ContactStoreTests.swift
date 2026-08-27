// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct ContactStoreTests {
    @Test
    func deleteAddressNamesRemovesContactName() throws {
        let test = try setupTest(
            chain: .bitcoin,
            address: "bc1qml9s2f9k8wc0882x63lyplzp97srzg2c39fyaw",
            addressType: .contact,
        )
        try test.addressStore.deleteAddressNames([test.addressName])
        #expect(try test.addressStore.getAddressName(chain: test.chain, address: test.address) == nil)
    }

    @Test
    func deleteAddressNamesPreservesOtherTypes() throws {
        let test = try setupTest(
            chain: .ethereum,
            address: "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7",
            addressType: .contract,
        )
        try test.addressStore.deleteAddressNames([AddressName.mock(chain: test.chain, address: test.address, name: test.contact.name, type: .contact)])
        #expect(try test.addressStore.getAddressName(chain: test.chain, address: test.address) == test.addressName)
    }

    @Test
    func updateContactDropsRemovedAddresses() throws {
        let test = try setupTest(
            chain: .bitcoin,
            address: "bc1qml9s2f9k8wc0882x63lyplzp97srzg2c39fyaw",
            addressType: .contact,
        )
        let addresses = try test.contactStore.getAddresses(contactId: test.contact.id)
        #expect(addresses.map(\.address) == [test.address])

        try test.contactStore.updateContact(test.contact, deleteAddressIds: addresses.map(\.id), addresses: [])
        #expect(try test.contactStore.getAddresses(contactId: test.contact.id).isEmpty)
    }

    private func setupTest(
        chain: Chain,
        address: String,
        addressType: AddressType,
    ) throws -> (contactStore: ContactStore, addressStore: AddressStore, contact: Contact, chain: Chain, address: String, addressName: AddressName) {
        let db = DB.mockWithChains([chain])
        let contactStore = ContactStore(db: db)
        let addressStore = AddressStore(db: db)
        let contact = Contact.mock()
        let addressName = AddressName.mock(chain: chain, address: address, name: contact.name, type: addressType)

        try contactStore.addContact(contact, addresses: [
            .mock(contactId: contact.id, address: address, chain: chain),
        ])
        try addressStore.addAddressNames([addressName])

        return (contactStore, addressStore, contact, chain, address, addressName)
    }
}
