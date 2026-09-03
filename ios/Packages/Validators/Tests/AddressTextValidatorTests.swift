// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemNameService
import GemstonePrimitivesTestKit
import PrimitivesTestKit
import Testing
@testable import Validators

struct AddressTextValidatorTests {
    let validETH = "0xb8c77482e45f1f44de1745f52c74426c631bdd52"
    let validBTC = "bc1qhgxl7yjhaazdhrfh0tzge572wkyp43h7t64fal"
    let nameService = GemNameService.mock()

    @Test
    func validatesCorrectAddress() throws {
        let eth = AddressTextValidator(asset: .mockBNB(), nameService: nameService)
        try eth.validate(validETH)

        let btc = AddressTextValidator(asset: .mock(), nameService: nameService)
        try btc.validate(validBTC)
    }

    @Test
    func throwsOnInvalidAddress() {
        let validator = AddressTextValidator(asset: .mock(), nameService: nameService)

        #expect(throws: TransferError.invalidAddress(asset: .mock())) {
            try validator.validate("not-an-address")
        }
    }
}
