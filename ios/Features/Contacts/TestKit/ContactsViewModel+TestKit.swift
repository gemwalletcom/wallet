// Copyright (c). Gem Wallet. All rights reserved.

import Contacts
import class Gemstone.GemContactService
import GemstonePrimitivesTestKit

public extension ContactsViewModel {
    @MainActor
    static func mock(mode: Mode = .list) -> ContactsViewModel {
        ContactsViewModel(
            service: GemContactService.mock(),
            contactEditor: { .mock(mode: $0) },
            mode: mode,
        )
    }
}
