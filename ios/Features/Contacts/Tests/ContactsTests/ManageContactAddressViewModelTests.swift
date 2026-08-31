// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemChainService
import class Gemstone.GemAddressService
import GemstonePrimitivesTestKit
import protocol Gemstone.GemNameServiceProtocol
import Components
@testable import Contacts
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct ManageContactAddressViewModelTests {
    @Test
    func buttonStateAddMode() {
        let model = ManageContactAddressViewModel.mock(mode: .add)

        #expect(model.buttonState == .disabled)

        model.addressInputModel.update(text: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")

        #expect(model.buttonState == .normal)
    }

    @Test
    func buttonStateEditMode() {
        let model = ManageContactAddressViewModel.mock(mode: .edit(.mock(address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")))

        #expect(model.buttonState == .normal)

        model.addressInputModel.update(text: "")

        #expect(model.buttonState == .disabled)
    }

    @Test
    func showMemo() {
        let model = ManageContactAddressViewModel.mock(mode: .add)

        model.addressInputModel.chain = .bitcoin
        #expect(model.showMemo == false)

        model.addressInputModel.chain = .cosmos
        #expect(model.showMemo == true)
    }

    @Test
    func nameResolveState() {
        let model = ManageContactAddressViewModel.mock(mode: .add)
        model.addressInputModel.text = "john"

        model.addressInputModel.nameRecordViewModel.state = .loading
        #expect(model.buttonState == .disabled)

        model.addressInputModel.nameRecordViewModel.state = .error
        #expect(model.buttonState == .disabled)

        model.addressInputModel.nameRecordViewModel.state = .complete(.mock(name: "john", chain: .bitcoin, address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"))
        #expect(model.buttonState == .normal)

        model.onSelectChain(.bitcoin)
        #expect(model.addressInputModel.nameRecordViewModel.state == .none)
    }
}

// MARK: - Mock

extension ManageContactAddressViewModel {
    static func mock(
        defaultChain: Chain = .bitcoin,
        nameService: any GemNameServiceProtocol = GemNameServiceMock(nameRecord: .mock()),
        mode: Mode,
    ) -> ManageContactAddressViewModel {
        ManageContactAddressViewModel(
            defaultChain: defaultChain,
            nameService: nameService,
            mode: mode,
            addressService: GemAddressService(),
            chainService: GemChainService(),
            onComplete: { _ in },
        )
    }
}
