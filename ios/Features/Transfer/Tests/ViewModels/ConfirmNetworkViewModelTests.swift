// Copyright (c). Gem Wallet. All rights reserved.

import Localization
@testable import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmNetworkViewModelTests {
    @Test
    func transfer() {
        let asset = Asset.mock()
        let model = ConfirmNetworkViewModel(type: .transfer(asset))

        guard case let .network(item) = model.itemModel else { return }
        #expect(item.title == Localized.Transfer.network)
        #expect(item.subtitle != nil)
        #expect(item.imageStyle != nil)
    }
}
