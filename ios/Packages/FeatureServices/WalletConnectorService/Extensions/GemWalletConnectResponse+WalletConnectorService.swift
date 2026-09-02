// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemWalletConnectResponse
import ReownWalletKit

extension GemWalletConnectResponse {
    func map() -> RPCResult {
        switch self {
        case let .response(value): .response(value.map())
        case .null: .response(AnyCodable.null())
        case let .error(error): .error(JSONRPCError(code: Int(error.code), message: error.message))
        }
    }
}
