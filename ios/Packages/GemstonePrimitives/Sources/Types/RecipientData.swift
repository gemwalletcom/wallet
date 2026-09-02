// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemRecipient
import Primitives

public struct RecipientData: Equatable, Hashable, Sendable {
    public let recipient: GemRecipient
    public let amount: String?

    public init(
        recipient: GemRecipient,
        amount: String?,
    ) {
        self.recipient = recipient
        self.amount = amount
    }
}

public struct PerpetualRecipientData: Equatable, Hashable, Sendable {
    public let recipient: RecipientData
    public let positionAction: PerpetualPositionAction

    public init(recipient: RecipientData, positionAction: PerpetualPositionAction) {
        self.recipient = recipient
        self.positionAction = positionAction
    }
}

extension PerpetualRecipientData: Identifiable {
    public var id: String {
        positionAction.id
    }
}
