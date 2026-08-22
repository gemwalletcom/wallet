// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public struct TransferDataExtra: Equatable, Sendable {
    public let to: String
    public let gasLimit: BigInt?
    public let gasPrice: GasPriceType?
    public let data: Data?
    public let outputType: TransferDataOutputType
    public let outputAction: TransferDataOutputAction
    public let transactionType: TransactionType
    public let approval: ApprovalData?

    public init(
        to: String,
        gasLimit: BigInt? = .none,
        gasPrice: GasPriceType? = .none,
        data: Data? = .none,
        outputType: TransferDataOutputType = .encodedTransaction,
        outputAction: TransferDataOutputAction = .send,
        transactionType: TransactionType = .smartContractCall,
        approval: ApprovalData? = nil,
    ) {
        self.to = to
        self.gasLimit = gasLimit
        self.gasPrice = gasPrice
        self.data = data
        self.outputType = outputType
        self.outputAction = outputAction
        self.transactionType = transactionType
        self.approval = approval
    }
}

extension TransferDataExtra: Hashable {}
