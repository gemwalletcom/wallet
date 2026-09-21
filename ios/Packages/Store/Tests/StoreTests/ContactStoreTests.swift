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
        let addressStore = AddressStore.mock(db: .mockWithChains([.bitcoin]))
        let addressName = AddressName.mock(chain: .bitcoin, address: "bc1qml9s2f9k8wc0882x63lyplzp97srzg2c39fyaw", type: .contact)
        try addressStore.updateAddressNames([.mock(addressName)])

        try addressStore.deleteAddressNames([addressName])

        #expect(try addressStore.getAddressName(chain: addressName.chain, address: addressName.address) == nil)
    }

    @Test
    func deleteAddressNamesPreservesOtherTypes() throws {
        let addressStore = AddressStore.mock(db: .mockWithChains([.ethereum]))
        let addressName = AddressName.mock(chain: .ethereum, address: "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7", type: .contract)
        try addressStore.updateAddressNames([.mock(addressName)])

        try addressStore.deleteAddressNames([.mock(chain: addressName.chain, address: addressName.address, type: .contact)])

        #expect(try addressStore.getAddressName(chain: addressName.chain, address: addressName.address) == addressName)
    }

    @Test
    func updateAddressNamesRenamesContactName() throws {
        let addressStore = AddressStore.mock(db: .mockWithChains([.bitcoin]))
        let addressName = AddressName.mock(chain: .bitcoin, address: "bc1qml9s2f9k8wc0882x63lyplzp97srzg2c39fyaw", type: .contact)
        try addressStore.updateAddressNames([.mock(addressName)])
        let renamed = AddressName.mock(chain: addressName.chain, address: addressName.address, name: "Bob", type: .contact)

        try addressStore.updateAddressNames([.mock(renamed)])

        #expect(try addressStore.getAddressName(chain: addressName.chain, address: addressName.address) == renamed)
    }

    @Test
    func updateAddressNamesKeepsRemoteNamesFromRenamingContacts() throws {
        let addressStore = AddressStore.mock(db: .mockWithChains([.bitcoin]))
        let addressName = AddressName.mock(chain: .bitcoin, address: "bc1qml9s2f9k8wc0882x63lyplzp97srzg2c39fyaw", type: .contact)
        try addressStore.updateAddressNames([.mock(addressName)])
        let remote = AddressName.mock(chain: addressName.chain, address: addressName.address, name: "Binance", type: .address)

        try addressStore.updateAddressNames([.mock(remote)])

        #expect(try addressStore.getAddressName(chain: addressName.chain, address: addressName.address) == addressName)
    }

    @Test
    func updateAddressNamesKeepsNamesReservedByAnotherLocalType() throws {
        let addressStore = AddressStore.mock(db: .mockWithChains([.bitcoin]))
        let addressName = AddressName.mock(chain: .bitcoin, address: "bc1qml9s2f9k8wc0882x63lyplzp97srzg2c39fyaw", type: .internalWallet)
        try addressStore.updateAddressNames([.mock(addressName)])
        let contact = AddressName.mock(chain: addressName.chain, address: addressName.address, name: "Bob", type: .contact)

        try addressStore.updateAddressNames([.mock(contact)])

        #expect(try addressStore.getAddressName(chain: addressName.chain, address: addressName.address) == addressName)
    }

    @Test
    func updateContactDropsRemovedAddresses() throws {
        let contactStore = ContactStore.mock(db: .mockWithChains([.bitcoin]))
        let contact = Contact.mock()
        let address = ContactAddress.mock(contactId: contact.id, address: "bc1qml9s2f9k8wc0882x63lyplzp97srzg2c39fyaw", chain: .bitcoin)
        try contactStore.addContact(contact, addresses: [address])

        let addresses = try contactStore.getAddresses(contactId: contact.id)
        #expect(addresses.map(\.address) == [address.address])

        try contactStore.updateContact(contact, deleteAddressIds: addresses.map(\.id), addresses: [])
        #expect(try contactStore.getAddresses(contactId: contact.id).isEmpty)
    }
}
