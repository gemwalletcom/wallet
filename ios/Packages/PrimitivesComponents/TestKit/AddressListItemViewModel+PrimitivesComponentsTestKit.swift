// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAddressFormatStyle
import Primitives
import PrimitivesComponents
import PrimitivesTestKit

public extension AddressListItemViewModel {
    static func mock(
        account: SimpleAccount = .mock(),
        mode: Mode = .auto(addressStyle: .short),
    ) -> AddressListItemViewModel {
        AddressListItemViewModel(
            title: "Recipient",
            account: account,
            mode: mode,
            addressLink: .mock(),
        )
    }
}
