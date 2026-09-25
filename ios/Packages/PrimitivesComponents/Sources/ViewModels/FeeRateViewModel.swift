// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemFeeAmount
import enum Gemstone.GemLocalizedText
import Localization
import Primitives
import Style
import SwiftUI

public struct FeeRateViewModel: Identifiable {
    public let priority: FeePriority
    public let value: GemLocalizedText
    public let fee: GemFeeAmount?
    public let isSelected: Bool

    public init(
        priority: FeePriority,
        value: GemLocalizedText,
        fee: GemFeeAmount?,
        isSelected: Bool,
    ) {
        self.priority = priority
        self.value = value
        self.fee = fee
        self.isSelected = isSelected
    }

    public var id: String {
        priority.rawValue
    }

    public var emoji: String {
        switch priority {
        case .fast: Emoji.FeeRate.fast.rawValue
        case .normal: Emoji.FeeRate.normal.rawValue
        }
    }

    public var title: String {
        priority.title
    }

    public var valueText: String {
        value.text
    }
}
