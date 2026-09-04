// Copyright (c). Gem Wallet. All rights reserved.

import Components
@testable import Contacts
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
    static func mock(mode: Mode) -> ManageContactViewModel {
        ManageContactViewModel(service: GemManageContactServiceMock(), nameService: GemNameServiceMock(), mode: mode)
    }
}
