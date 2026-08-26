// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension Primitives.Account {
    func map() -> Gemstone.Account {
        Gemstone.Account(
            chain: chain.rawValue,
            address: address,
            derivationPath: derivationPath,
            extendedPublicKey: extendedPublicKey,
        )
    }
}

public extension TransferData {
    func confirmInput(from account: Primitives.Account, available: BigInt) throws -> GemConfirmInput {
        try GemConfirmInput(
            inputType: type.map(),
            from: account.map(),
            destination: GemConfirmDestination(
                address: recipientData.recipient.address,
                name: recipientData.recipient.name,
            ),
            value: value.description,
            memo: recipientData.recipient.memo,
            references: recipientData.recipient.references,
            useMax: available > 0 && available == value,
            minimumValue: minimumValue?.description,
        )
    }
}

public extension FeeSelection {
    func map() -> GemConfirmFeeSelection {
        switch self {
        case let .preset(priority): .priority(priority: priority.rawValue)
        case let .custom(gasPrice): .custom(gasPrice: gasPrice.description)
        }
    }
}

public extension Gemstone.ScanTransactionPayload {
    func map() throws -> Primitives.ScanTransactionPayload {
        try Primitives.ScanTransactionPayload(
            origin: origin.map(),
            target: target.map(),
            website: website,
            type: transactionType.map(),
        )
    }
}

public extension Gemstone.ScanAddressTarget {
    func map() throws -> Primitives.ScanAddressTarget {
        try Primitives.ScanAddressTarget(
            assetId: AssetId(id: assetId),
            address: address,
        )
    }
}

public extension Primitives.ScanTransaction {
    func map() -> Gemstone.ScanTransaction {
        Gemstone.ScanTransaction(
            isMalicious: isMalicious,
            isMemoRequired: isMemoRequired,
        )
    }
}
