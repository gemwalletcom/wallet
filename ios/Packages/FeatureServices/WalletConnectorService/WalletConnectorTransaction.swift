// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public enum WalletConnectorEVMTransactionKind {
    case transfer
    case contractCall
    case tokenApproval(ApprovalData)
}

public extension WalletConnectorEVMTransactionKind {
    var transactionType: TransactionType {
        switch self {
        case .transfer: .transfer
        case .contractCall: .smartContractCall
        case .tokenApproval: .tokenApproval
        }
    }

    var approvalData: ApprovalData? {
        switch self {
        case .transfer, .contractCall: .none
        case let .tokenApproval(approval): approval
        }
    }
}

public enum WalletConnectorTransaction {
    case ethereum(WCEthereumTransaction, WalletConnectorEVMTransactionKind)
    case solana(String, TransferDataOutputType, TransactionType)
    case sui(String, TransferDataOutputType)
    case ton(String, TransferDataOutputType)
    case tron(String, TransferDataOutputType)
}

public extension WalletConnectorTransaction {
    var transactionType: TransactionType {
        switch self {
        case let .ethereum(_, kind): kind.transactionType
        case let .solana(_, _, transactionType): transactionType
        case .sui, .ton, .tron: .smartContractCall
        }
    }
}
