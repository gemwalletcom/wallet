// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSwapProgressMarker
import struct Gemstone.GemSwapProgressState
import enum Gemstone.GemSwapProgressStep
import Localization
import Style
import SwiftUI

public struct TransactionSwapProgressItemModel: Equatable {
    public struct Step: Equatable {
        public let title: String
        public let subtitle: String
        public let state: GemSwapProgressState

        public init(
            title: String,
            subtitle: String,
            state: GemSwapProgressState,
        ) {
            self.title = title
            self.subtitle = subtitle
            self.state = state
        }
    }

    public let transfer: Step
    public let swap: Step
    public let estimatedTime: String?

    public init(
        transfer: Step,
        swap: Step,
        estimatedTime: String?,
    ) {
        self.transfer = transfer
        self.swap = swap
        self.estimatedTime = estimatedTime
    }
}

extension GemSwapProgressStep {
    var color: Color {
        switch self {
        case .completed: Colors.green
        case .pending: Colors.blue
        case .waiting: Colors.gray
        case .failed, .reverted: Colors.red
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
}

extension GemSwapProgressState {
    var color: Color {
        step.color
    }

    var background: Color {
        step.background
    }

    var lineColor: Color {
        step.lineColor
    }

    var markerBackground: Color {
        switch marker {
        case .check, .cross, .swap: background
        case .spinner, .dots: .clear
        }
    }
}
