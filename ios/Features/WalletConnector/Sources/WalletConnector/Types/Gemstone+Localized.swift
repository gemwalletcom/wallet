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
