// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.SignMessage
import Primitives
import SigningRequestService

public enum PaymentAction: Sendable {
    case signMessage(chain: Chain, message: SignMessage)
    case signTransaction(chain: Chain, transaction: SignableTransaction)
    case sendTransaction(chain: Chain, transaction: SignableTransaction)
    case approveToken(chain: Chain, approval: ApprovalData)
}
