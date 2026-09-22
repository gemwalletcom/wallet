// Copyright (c). Gem Wallet. All rights reserved.

import Contacts
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit

public extension ContactEditorViewModel {
    @MainActor
    static func mock(mode: Mode = .add()) -> ContactEditorViewModel {
        ContactEditorViewModel(
            service: GemContactEditorServiceMock(),
            nameService: GemNameServiceMock(),
            mode: mode,
        )
    }
}
