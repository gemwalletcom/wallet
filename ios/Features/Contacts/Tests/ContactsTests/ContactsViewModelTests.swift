// Copyright (c). Gem Wallet. All rights reserved.

@testable import Contacts
import ContactsTestKit
import Gemstone
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct ContactsViewModelTests {
    @Test
    func theListPicksARowAndTheAddressFlowPicksAContact() {
        let list = ContactsViewModel.mock()
        let picking = ContactsViewModel.mock(mode: .addAddress(.mock(address: "bc1qar0"), chain: .bitcoin))

        #expect(list.rowAction == .navigate)
        #expect(picking.rowAction == .select)

        guard case .add(.none, .none) = list.addContactMode else {
            Issue.record("the list adds an empty contact")
            return
        }
        guard case let .add(recipient, chain) = picking.addContactMode else {
            Issue.record("the address flow seeds the contact it is adding to")
            return
        }
        #expect(recipient?.address == "bc1qar0")
        #expect(chain == .bitcoin)
    }

    @Test
    func aRowReadsItsTitleAndSubtitleFromCore() {
        let model = ContactsViewModel.mock()
        let row = model.listItemModel(for: .mock(contact: .mock(name: "Satoshi"), addresses: [.mock()]))

        #expect(row.title == "Satoshi")
    }
}
