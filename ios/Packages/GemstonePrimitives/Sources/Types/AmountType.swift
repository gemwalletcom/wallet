// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemPaymentRecipient
import enum Gemstone.GemPerpetualPositionAction
import enum Gemstone.GemStakeAmountInput
import Primitives

public enum AmountType: Equatable, Hashable, Sendable {
    case transfer(recipient: GemPaymentRecipient)
    case deposit
    case withdraw
    case stake(GemStakeAmountInput)
    case perpetual(GemPerpetualPositionAction)
    case earn(EarnType)
}
