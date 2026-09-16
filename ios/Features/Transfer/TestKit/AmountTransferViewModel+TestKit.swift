// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAmountService
import enum Gemstone.GemAmountTransfer
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
@testable import Transfer

public extension AmountTransferViewModel {
    static func mock(
        asset: Asset = .mock(),
        transfer: GemAmountTransfer = .send(payment: .mock()),
    ) -> AmountTransferViewModel {
        AmountTransferViewModel(asset: asset, transfer: transfer, service: GemAmountService.mock())
    }
}
