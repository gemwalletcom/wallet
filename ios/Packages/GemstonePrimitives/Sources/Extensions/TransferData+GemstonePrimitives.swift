// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemRecipient
import struct Gemstone.GemTransferBalance
import struct Gemstone.GemTransferData
import class Gemstone.GemTransferService
import Primitives

public extension TransferData {
    init(_ transfer: GemTransferData) throws {
        self.init(
            type: try transfer.inputType.map(),
            recipient: Recipient(transfer.recipient),
            value: try BigInt.from(string: transfer.value),
            useMaxAmount: transfer.useMaxAmount,
            minimumValue: try transfer.minimumValue.map { try BigInt.from(string: $0) },
        )
    }

    var gem: GemTransferData {
        GemTransferData(
            inputType: type.inputType,
            recipient: recipient.gem,
            value: value.description,
            useMaxAmount: useMaxAmount,
            minimumValue: minimumValue?.description,
        )
    }

    func availableValue(balance: Balance, transferService: GemTransferService) throws -> BigInt {
        let transferBalance = GemTransferBalance(
            available: balance.available.description,
            frozen: balance.frozen.description,
            locked: balance.locked.description,
            withdrawable: balance.withdrawable.description,
            votes: UInt32(balance.metadata?.votes ?? 0),
        )
        return try BigInt.from(string: transferService.availableValue(transfer: gem, balance: transferBalance))
    }
}

public extension Recipient {
    init(_ recipient: GemRecipient) {
        self.init(name: recipient.name, address: recipient.address, memo: recipient.memo, references: recipient.references)
    }

    var gem: GemRecipient {
        GemRecipient(address: address, name: name, memo: memo, references: references)
    }
}
