// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

@MainActor
struct AddressInputViewModelTests {
    @Test
    func validate() {
        let model = AddressInputViewModel.mock()

        model.inputModel.text = "gemcoder"
        #expect(model.validate() == false)

        model.inputModel.text = "test.eth"
        model.nameRecordViewModel.state = .loading(name: "test.eth", chain: Chain.ethereum.toGem())
        #expect(model.validate() == false)

        model.nameRecordViewModel.state = .error
        #expect(model.validate() == false)

        model.nameRecordViewModel.state = .complete(record: NameRecord.mock(name: "test.eth", chain: .ethereum, address: "0x1234567890123456789012345678901234567890").toGem())
        #expect(model.validate())

        model.nameRecordViewModel.state = .complete(record: NameRecord.mock(name: "other.eth", chain: .ethereum, address: "0x1234567890123456789012345678901234567890").toGem())
        #expect(model.validate() == false)

        model.nameRecordViewModel.state = .complete(record: NameRecord.mock(name: "test.eth", chain: .ethereum, address: "test.eth").toGem())
        #expect(model.validate() == false)

        model.chain = .near
        model.inputModel.text = "h3rman.near"
        model.nameRecordViewModel.state = .complete(record: NameRecord.mock(name: "h3rman.near", chain: .near, address: "h3rman.near", provider: .near).toGem())
        #expect(model.validate())
    }

    @Test
    func aNameStillResolvingIsNotYetAnError() {
        let model = AddressInputViewModel.mock()

        model.inputModel.text = "test.eth"
        model.nameRecordViewModel.state = .loading(name: "test.eth", chain: Chain.ethereum.toGem())

        #expect(model.validate() == false)
        #expect(model.inputModel.error == nil, "a name the resolver still owns must not read as a bad address")

        model.nameRecordViewModel.state = .complete(record: NameRecord.mock(name: "test.eth", chain: .ethereum, address: "0x1234567890123456789012345678901234567890").toGem())
        #expect(model.validate())
        #expect(model.inputModel.error == nil)
    }

    @Test
    func anAddressThatIsNotOneShowsTheError() {
        let model = AddressInputViewModel.mock()

        model.inputModel.text = "gemcoder"
        #expect(model.validate() == false)
        #expect(model.inputModel.error != nil)

        model.inputModel.text = ""
        #expect(model.validate())
        #expect(model.inputModel.error == nil, "an empty field is silent, the way Core and Android treat it")

        model.inputModel.text = "0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a"
        #expect(model.validate())
        #expect(model.inputModel.error == nil)
    }

    @Test
    func chainChangeResetsState() {
        let model = AddressInputViewModel.mock()

        model.inputModel.text = "sometext"
        model.nameRecordViewModel.state = .complete(record: NameRecord.mock(name: "test.eth", chain: .ethereum, address: "0x1234567890123456789012345678901234567890").toGem())
        model.chain = .bitcoin

        #expect(model.nameResolveState == .none)
        #expect(model.text == "sometext")
    }

    @Test
    func resolvedAddressUsesChecksumAddress() {
        let model = AddressInputViewModel.mock()
        let address = "0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a"
        let checksummed = "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a"

        model.inputModel.text = " \n\(address)\r "
        #expect(model.resolvedAddress == checksummed)

        model.nameRecordViewModel.state = .complete(record: NameRecord.mock(name: "test.eth", chain: .ethereum, address: address).toGem())
        #expect(model.resolvedAddress == checksummed)
    }
}
