// Copyright (c). Gem Wallet. All rights reserved.

import Contacts
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit

public extension ManageContactViewModel {
    @MainActor
    static func mock(mode: Mode = .add()) -> ManageContactViewModel {
        ManageContactViewModel(
            service: GemManageContactServiceMock(),
            nameService: GemNameServiceMock(),
            mode: mode,
        )
    }
}
