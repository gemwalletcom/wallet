// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.PaymentAction
import GemstonePrimitives
import Primitives
import SigningRequestService

typealias GemPaymentAction = Gemstone.PaymentAction

public extension GemPaymentAction {
    func map() throws -> PaymentAction {
        switch self {
        case let .signMessage(message):
            try .signMessage(chain: message.chain.map(), message: message)
        case let .signTransaction(chain, transaction):
            try .signTransaction(chain: chain.map(), transaction: transaction.map())
        case let .sendTransaction(chain, transaction):
            try .sendTransaction(chain: chain.map(), transaction: transaction.map())
        case let .approveToken(chain, approval):
            try .approveToken(chain: chain.map(), approval: approval.map())
        }
    }
}
