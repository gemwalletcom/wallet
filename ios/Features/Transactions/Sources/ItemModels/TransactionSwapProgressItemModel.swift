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

        public var color: Color { state.color }
        public var markerBackground: Color { state.markerBackground }
        public var lineColor: Color { state.lineColor }
        public var tagBackground: Color { state.background }
        public var tagTitle: String? { state.step.tagTitle }
        public var markerImage: Image? { state.marker.image }
        public var showsSpinner: Bool { state.marker == .spinner }

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
