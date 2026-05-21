// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Style
import SwiftUI

public struct TransactionSwapProgressItemModel: Equatable {
    public struct Step: Equatable {
        public enum Status: Equatable {
            case completed
            case pending
            case waiting
            case failed
            case reverted
            case refunded
        }

        public let title: String
        public let subtitle: String
        public let status: Status

        public init(
            title: String,
            subtitle: String,
            status: Status,
        ) {
            self.title = title
            self.subtitle = subtitle
            self.status = status
        }
    }

    public let transfer: Step
    public let swap: Step

    public init(
        transfer: Step,
        swap: Step,
    ) {
        self.transfer = transfer
        self.swap = swap
    }
}

extension TransactionSwapProgressItemModel.Step.Status {
    var tagTitle: String? {
        switch self {
        case .completed: Localized.Transaction.Status.completed
        case .pending: Localized.Transaction.Status.inprogress
        case .waiting: nil
        case .failed: Localized.Transaction.Status.failed
        case .reverted: Localized.Transaction.Status.reverted
        case .refunded: Localized.Transaction.Status.refunded
        }
    }

    var color: Color {
        switch self {
        case .completed: Colors.green
        case .pending: Colors.blue
        case .waiting: Colors.gray
        case .failed: Colors.red
        case .reverted: Colors.red
        case .refunded: Colors.orange
        }
    }

    var background: Color {
        color.opacity(.light)
    }

    var lineColor: Color {
        switch self {
        case .completed: Colors.green
        case .pending, .waiting, .failed, .reverted, .refunded: Colors.gray.opacity(.medium)
        }
    }

    var markerBackground: Color {
        switch self {
        case .completed, .failed, .reverted, .refunded: background
        case .pending, .waiting: .clear
        }
    }
}
