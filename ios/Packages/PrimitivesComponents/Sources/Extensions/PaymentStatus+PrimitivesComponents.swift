// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives

public extension PaymentStatus {
    var title: String {
        switch self {
        case .succeeded: Localized.Transaction.Status.confirmed
        case .processing, .requiresAction: Localized.Transaction.Status.pending
        case .failed, .expired, .cancelled: Localized.Transaction.Status.failed
        }
    }
}
