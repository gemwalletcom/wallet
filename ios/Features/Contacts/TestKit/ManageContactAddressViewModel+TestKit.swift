// Copyright (c). Gem Wallet. All rights reserved.

import Contacts
import GemstonePrimitivesTestKit

public extension ManageContactAddressViewModel {
    @MainActor
    static func mock(contactId: String = "contact", mode: Mode = .add) -> ManageContactAddressViewModel {
        ManageContactAddressViewModel(
            service: GemManageContactServiceMock(),
            nameService: GemNameServiceMock(),
            contactId: contactId,
            mode: mode,
            onComplete: { _ in },
        )
    }
}
