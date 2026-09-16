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
        state.statusTitle
    }

    public var description: String {
        tone.infoDescription
    }

    public var stateImage: Image {
        tone.image
    }

    public var color: Color {
        tone.color
    }

    public var background: Color {
        color.opacity(.light)
    }
}
