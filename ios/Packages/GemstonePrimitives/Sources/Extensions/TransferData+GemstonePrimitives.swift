// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GemTransactionInputType
import struct Gemstone.GemRecipient
import struct Gemstone.GemTransferBalance
import struct Gemstone.GemTransferData
import class Gemstone.GemTransferService
import Primitives

public extension GemTransferData {
    init(
        inputType: GemTransactionInputType,
        recipient: GemRecipient,
        value: BigInt,
        useMaxAmount: Bool = false,
        minimumValue: BigInt? = nil,
    ) {
        self.init(
            inputType: inputType,
            recipient: recipient,
            value: value.description,
            useMaxAmount: useMaxAmount,
            minimumValue: minimumValue?.description,
        )
    }

    var chain: Chain {
        inputType.chain
    }

    var id: String {
        [chain.rawValue, recipient.address, value].joined(separator: "-")
    }

    func availableValue(balance: Balance, transferService: GemTransferService) throws -> BigInt {
        let transferBalance = GemTransferBalance(
            available: balance.available.description,
            frozen: balance.frozen.description,
            locked: balance.locked.description,
            withdrawable: balance.withdrawable.description,
            votes: UInt32(balance.metadata?.votes ?? 0),
        )
        return try BigInt.from(string: transferService.availableValue(transfer: self, balance: transferBalance))
    }
}

