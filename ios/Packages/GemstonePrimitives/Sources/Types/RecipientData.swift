// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemPerpetualPositionAction
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
    public let positionAction: GemPerpetualPositionAction

    public init(recipient: RecipientData, positionAction: GemPerpetualPositionAction) {
        self.recipient = recipient
        self.positionAction = positionAction
    }
}

extension PerpetualRecipientData: Identifiable {
    public var id: String {
        let data = positionAction.transferData()
        return "\(data.assetIndex)_\(data.direction)"
    }
}
