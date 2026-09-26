// Copyright (c). Gem Wallet. All rights reserved.

import Contacts
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit

public extension ContactEditorSceneViewModel {
    @MainActor
    static func mock(mode: Mode = .add()) -> ContactEditorSceneViewModel {
        ContactEditorSceneViewModel(
            service: GemContactEditorServiceMock(),
            nameService: GemNameServiceMock(),
            mode: mode,
        )
    }
}
