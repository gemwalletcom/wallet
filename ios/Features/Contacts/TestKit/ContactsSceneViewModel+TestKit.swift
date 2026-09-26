// Copyright (c). Gem Wallet. All rights reserved.

import Contacts
import class Gemstone.GemContactService
import GemstonePrimitivesTestKit

public extension ContactsSceneViewModel {
    @MainActor
    static func mock(mode: Mode = .list) -> ContactsSceneViewModel {
        ContactsSceneViewModel(
            service: GemContactService.mock(),
            contactEditor: { .mock(mode: $0) },
            mode: mode,
        )
    }
}
