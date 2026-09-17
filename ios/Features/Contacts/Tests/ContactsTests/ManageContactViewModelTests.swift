// Copyright (c). Gem Wallet. All rights reserved.

@testable import Contacts
import ContactsTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import GemstonePrimitivesTestKit
import PrimitivesTestKit
import Testing

@MainActor
struct ManageContactViewModelTests {
    @Test
    func buttonStateAddMode() {
        let model = ManageContactViewModel.mock()

        #expect(model.buttonState == .disabled)

        model.nameInputModel.text = "John"

        #expect(model.buttonState == .normal)
    }

    @Test
    func buttonStateEditMode() {
        let model = ManageContactViewModel.mock(mode: .edit(.mock(contact: .mock(name: "John"), addresses: [.mock()])))

        #expect(model.buttonState == .normal)

        model.nameInputModel.text = ""

        #expect(model.buttonState == .disabled)
    }
}
