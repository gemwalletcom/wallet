// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemVerificationLevel
import enum Gemstone.MessageType
import GemstonePrimitives
import Localization
import Primitives

extension GemVerificationLevel {
    var title: String {
        switch self {
        case .verified: Localized.Asset.Verification.verified
        case .unverified: Localized.Asset.Verification.unverified
        case .suspicious: Localized.Asset.Verification.suspicious
        }
    }
}

extension MessageType {
    var title: String {
        switch self {
        case .siwe: Localized.Common.signInWith(Chain.ethereum.networkName)
        case .siws: Localized.Common.signInWith(Chain.solana.networkName)
        case .text, .eip712: Localized.Transfer.reviewRequest
        }
    }
}
