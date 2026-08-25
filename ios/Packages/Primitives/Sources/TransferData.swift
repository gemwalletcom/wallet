// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public struct TransferData: Identifiable, Sendable, Hashable {
    public let type: TransferDataType
    public let recipientData: RecipientData
    public let amount: TransferAmountValue
    public let minimumValue: BigInt?

    public init(
        type: TransferDataType,
        recipientData: RecipientData,
        amount: TransferAmountValue,
        minimumValue: BigInt? = nil,
    ) {
        self.type = type
        self.recipientData = recipientData
        self.amount = amount
        self.minimumValue = minimumValue
    }

    public init(
        asset: Asset,
        metadata: ApplicationMetadata,
        transaction: String,
        memo: String?,
        outputType: TransferDataOutputType,
        outputAction: TransferDataOutputAction,
        transactionType: TransactionType,
    ) {
        self.init(
            type: .generic(
                asset: asset,
                metadata: metadata,
                extra: TransferDataExtra(
                    to: "",
                    data: Data(transaction.utf8),
                    outputType: outputType,
                    outputAction: outputAction,
                    transactionType: transactionType,
                ),
            ),
            recipientData: RecipientData(
                recipient: Recipient(name: nil, address: "", memo: memo),
                amount: nil,
            ),
            amount: .exact(.zero),
        )
    }

    public var value: BigInt {
        amount.value
    }

    public var id: String {
        [type.transactionType.rawValue, recipientData.recipient.address, String(value)].joined(separator: "-")
    }

    public var chain: Chain {
        type.chain
    }

    public func encodedTransaction() throws -> String {
        guard case let .generic(_, _, extra) = type,
              extra.outputType == .encodedTransaction,
              let data = extra.data
        else {
            throw AnyError("Missing encoded transaction")
        }
        return try data.encodeString()
    }
}
