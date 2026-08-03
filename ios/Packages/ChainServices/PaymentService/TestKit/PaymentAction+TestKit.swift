// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.SignMessage
import PaymentService
import Primitives

public extension PaymentAction {
    static func mockSignMessage(chain: Chain = .ethereum, data: Data = Data("pay".utf8)) -> PaymentAction {
        .signMessage(chain: chain, message: SignMessage(chain: chain.rawValue, signType: .eip712, data: data))
    }
}
