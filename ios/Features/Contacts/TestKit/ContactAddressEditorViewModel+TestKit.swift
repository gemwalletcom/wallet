// Copyright (c). Gem Wallet. All rights reserved.

import Contacts
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit

public extension ContactAddressEditorViewModel {
    @MainActor
    static func mock(contactId: String = "contact", mode: Mode = .add) -> ContactAddressEditorViewModel {
        ContactAddressEditorViewModel(
            service: GemContactEditorServiceMock(),
            nameService: GemNameServiceMock(),
            contactId: contactId,
            mode: mode,
            onComplete: { _ in },
        )
    }
}
