// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import PrimitivesComponentsTestKit
import Testing

struct AddressListItemViewModelTests {
    @Test
    func subtitleWithName() {
        let model = AddressListItemViewModel.mock()
        #expect(model.subtitle == "Alice (0x12345...01112)")
    }

    @Test
    func subtitleWithoutAddress() {
        let account = SimpleAccount.mock(assetImage: AssetImage())
        let model = AddressListItemViewModel.mock(account: account)
        #expect(model.subtitle == "Alice")
    }

    @Test
    func subtitleWithoutName() {
        let account = SimpleAccount.mock(name: nil)
        let model = AddressListItemViewModel.mock(account: account, mode: .auto(addressStyle: .full))
        #expect(model.subtitle == "0x123456789101112")
    }

    @Test
    func subtitleAddressMode() {
        let model = AddressListItemViewModel.mock(mode: .address(addressStyle: .short))
        #expect(model.subtitle == "0x12345...01112")
    }

    @Test
    func subtitleNameOrAddressWithName() {
        let model = AddressListItemViewModel.mock(mode: .nameOrAddress)
        #expect(model.subtitle == "Alice")
    }

    @Test
    func subtitleNameOrAddressWithoutName() {
        let account = SimpleAccount.mock(name: nil)
        let model = AddressListItemViewModel.mock(account: account, mode: .nameOrAddress)
        #expect(model.subtitle == "0x123456789101112")
    }
}
