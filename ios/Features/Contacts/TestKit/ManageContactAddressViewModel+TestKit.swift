// Copyright (c). Gem Wallet. All rights reserved.

import Contacts
import GemstonePrimitivesTestKit

public extension ManageContactAddressViewModel {
    @MainActor
    static func mock(mode: Mode = .add) -> ManageContactAddressViewModel {
        ManageContactAddressViewModel(
            service: GemManageContactServiceMock(),
            nameService: GemNameServiceMock(),
            mode: mode,
            onComplete: { _ in },
        )
    }
}
