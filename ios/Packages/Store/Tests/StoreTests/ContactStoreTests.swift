// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct ContactStoreTests {
    @Test
    func deleteContactRemovesAddressName() throws {
        let test = try setupTest(
            chain: .bitcoin,
            address: "bc1qml9s2f9k8wc0882x63lyplzp97srzg2c39fyaw",
            addressType: .contact,
        )

        try test.contactStore.deleteContact(id: test.contact.id)

        #expect(try test.addressStore.getAddressName(chain: test.chain, address: test.address) == nil)
    }

    @Test
    func deleteContactPreservesNonContactAddressName() throws {
        let test = try setupTest(
            chain: .ethereum,
            address: "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7",
            addressType: .contract,
        )

        try test.contactStore.deleteContact(id: test.contact.id)

        #expect(try test.addressStore.getAddressName(chain: test.chain, address: test.address) == test.addressName)
    }

    @Test
    func updateContactRemovesAddressNameForRemovedAddress() throws {
        let test = try setupTest(
            chain: .bitcoin,
            address: "bc1qml9s2f9k8wc0882x63lyplzp97srzg2c39fyaw",
            addressType: .contact,
        )
        let addressIds = try test.contactStore.getAddressIds(contactId: test.contact.id)

        try test.contactStore.updateContact(test.contact, deleteAddressIds: addressIds, addresses: [])

        #expect(try test.addressStore.getAddressName(chain: test.chain, address: test.address) == nil)
    }

    @Test
    func updateContactPreservesNonContactAddressName() throws {
        let test = try setupTest(
            chain: .ethereum,
            address: "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7",
            addressType: .contract,
        )
        let addressIds = try test.contactStore.getAddressIds(contactId: test.contact.id)

        try test.contactStore.updateContact(test.contact, deleteAddressIds: addressIds, addresses: [])

        #expect(try test.addressStore.getAddressName(chain: test.chain, address: test.address) == test.addressName)
    }
}

// MARK: - Private

extension ContactStoreTests {
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
