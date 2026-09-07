// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemTransactionStateTone
import Localization
import Primitives
import Style
import SwiftUI

public struct TransactionStateViewModel: Equatable, Sendable {
    public let state: TransactionState
    public let tone: GemTransactionStateTone

    public init(state: TransactionState, tone: GemTransactionStateTone) {
        self.state = state
        self.tone = tone
    }

    public var title: String {
        switch state {
        case .confirmed: Localized.Transaction.Status.confirmed
        case .pending, .inTransit: Localized.Transaction.Status.pending
        case .failed: Localized.Transaction.Status.failed
        case .reverted: Localized.Transaction.Status.reverted
        case .refunded: Localized.Transaction.Status.refunded
        }
    }

    public var description: String {
        switch tone {
        case .pending: Localized.Info.Transaction.Pending.description
        case .success: Localized.Info.Transaction.Success.description
        case .error, .refunded: Localized.Info.Transaction.Error.description
        }
    }

    public var stateImage: Image {
        switch tone {
        case .pending: Images.Transaction.State.pending
        case .success: Images.Transaction.State.success
        case .error, .refunded: Images.Transaction.State.error
        }
    }

    public var color: Color {
        switch tone {
        case .success: Colors.green
        case .pending, .refunded: Colors.orange
        case .error: Colors.red
        }
    }

    public var background: Color {
        color.opacity(.light)
    }
}
