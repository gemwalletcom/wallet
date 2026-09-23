// Copyright (c). Gem Wallet. All rights reserved.

@testable import Contacts
import ContactsTestKit
import class Gemstone.GemChainService
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct ContactAddressEditorViewModelTests {
    @Test
    func buttonStateAddMode() {
        let model = ContactAddressEditorViewModel.mock()

        #expect(model.buttonState == .disabled)

        model.addressInputModel.update(text: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")

        #expect(model.buttonState == .normal)
    }

    @Test
    func buttonStateEditMode() {
        let model = ContactAddressEditorViewModel.mock(mode: .edit(.mock(address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")))

        #expect(model.buttonState == .normal)

        model.addressInputModel.update(text: "")

        #expect(model.buttonState == .disabled)
    }

    @Test
    func memoFieldFollowsTheChain() {
        let model = ContactAddressEditorViewModel.mock()

        model.addressInputModel.chain = .bitcoin
        #expect(model.fields == [.network, .address])

        model.addressInputModel.chain = .cosmos
        #expect(model.fields == [.network, .address, .memo])
    }

    @Test
    func nameResolveState() {
        let model = ContactAddressEditorViewModel.mock()
        model.addressInputModel.text = "john"

        model.addressInputModel.nameRecordViewModel.state = .loading(name: "john", chain: Chain.ethereum.toGem())
        #expect(model.buttonState == .disabled)

        model.addressInputModel.nameRecordViewModel.state = .error
        #expect(model.buttonState == .disabled)

        model.addressInputModel.nameRecordViewModel.state = .complete(record: NameRecord.mock(name: "john", chain: .bitcoin, address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh").toGem())
        #expect(model.buttonState == .normal)

        model.onSelectChain(.bitcoin)
        #expect(model.addressInputModel.nameRecordViewModel.state == .none)
    }

    @Test
    func aPaymentUriFillsTheAddressAndMemo() {
        let model = ContactAddressEditorViewModel.mock()

        model.onScan("ripple:rLpq5RcRzA5FLmVp8jZmdvfMiRZ2xtVvZK?dt=5")

        #expect(model.addressInputModel.text == "rLpq5RcRzA5FLmVp8jZmdvfMiRZ2xtVvZK")
        #expect(model.memo == "5")
    }

    @Test
    func theNetworkPickerFollowsCoreChainOrder() {
        let model = ContactAddressEditorViewModel.mock()
        let chains = GemChainService.shared.getChains(query: .empty).map { Chain(core: $0) }

        #expect(model.networkSelectorModel.state.value?.items == chains)
        #expect(chains != Chain.allCases)
    }
}
