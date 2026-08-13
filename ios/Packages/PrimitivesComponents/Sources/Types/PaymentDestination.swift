// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public enum PaymentDestination: Sendable {
    case confirm(TransferData)
    case recipient(RecipientData)
}
