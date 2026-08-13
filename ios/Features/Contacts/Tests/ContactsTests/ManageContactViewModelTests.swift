// Copyright (c). Gem Wallet. All rights reserved.

import Components
@testable import Contacts
import ContactService
import Primitives
import PrimitivesTestKit
import StoreTestKit
import Testing

@MainActor
struct ManageContactViewModelTests {
    @Test
    func buttonStateAddMode() {
        let model = ManageContactViewModel.mock(mode: .add())

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

// MARK: - Mock

extension ManageContactViewModel {
    static func mock(
        nameService: any NameServiceable = .mock(),
        mode: Mode,
    ) -> ManageContactViewModel {
        ManageContactViewModel(
            service: ContactService(store: .mock(), addressStore: .mock()),
            nameService: nameService,
            mode: mode,
        )
    }
}
