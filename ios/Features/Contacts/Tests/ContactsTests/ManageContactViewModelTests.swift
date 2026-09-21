// Copyright (c). Gem Wallet. All rights reserved.

@testable import Contacts
import ContactsTestKit
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct ManageContactViewModelTests {
    @Test
    func buttonStateAddMode() {
        let model = ManageContactViewModel.mock()

        #expect(model.buttonState == .disabled)

        model.onChangeName("John")

        #expect(model.buttonState == .normal)
    }

    @Test
    func buttonStateEditMode() {
        let model = ManageContactViewModel.mock(mode: .edit(.mock(contact: .mock(name: "John"), addresses: [.mock()])))

        #expect(model.buttonState == .normal)

        model.onChangeName("")

        #expect(model.buttonState == .disabled)
    }
}
