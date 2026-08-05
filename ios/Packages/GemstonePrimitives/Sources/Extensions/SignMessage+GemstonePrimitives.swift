// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.SignMessage {
    func map() -> Primitives.SignMessage {
        Primitives.SignMessage(
            chain: chain,
            signType: signType.map(),
            data: data,
        )
    }
}

public extension Primitives.SignMessage {
    func map() -> Gemstone.SignMessage {
        Gemstone.SignMessage(
            chain: chain,
            signType: signType.map(),
            data: data,
        )
    }
}

public extension Gemstone.SignDigestType {
    func map() -> Primitives.SignDigestType {
        switch self {
        case .eip191: .eip191
        case .eip712: .eip712
        case .base58: .base58
        case .suiPersonal: .suiPersonal
        case .siwe: .siwe
        case .tonPersonal: .tonPersonal
        case .tronPersonal: .tronPersonal
        }
    }
}

public extension Primitives.SignDigestType {
    func map() -> Gemstone.SignDigestType {
        switch self {
        case .eip191: .eip191
        case .eip712: .eip712
        case .base58: .base58
        case .suiPersonal: .suiPersonal
        case .siwe: .siwe
        case .tonPersonal: .tonPersonal
        case .tronPersonal: .tronPersonal
        }
    }
}
