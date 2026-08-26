// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.EvmTransactionKind
import enum Gemstone.WalletConnectTransaction
import struct Gemstone.WcEthereumTransactionData
import struct Gemstone.WcSolanaTransactionData
import struct Gemstone.WcSuiTransactionData
import GemstonePrimitives
import Primitives

extension WalletConnectTransaction {
    func map() throws -> WalletConnectorTransaction {
        switch self {
        case let .ethereum(data, kind): try .ethereum(data.map(), kind.map())
        case let .solana(data, outputType, transactionType):
            try .solana(data.transaction, Primitives.TransferDataOutputType(outputType), Primitives.TransactionType(transactionType))
        case let .sui(data, outputType): try .sui(data.transaction, Primitives.TransferDataOutputType(outputType))
        case let .ton(data, outputType): try .ton(data, Primitives.TransferDataOutputType(outputType))
        case let .tron(data, outputType): try .tron(data, Primitives.TransferDataOutputType(outputType))
        }
    }
}

extension EvmTransactionKind {
    func map() throws -> WalletConnectorEVMTransactionKind {
        switch self {
        case .transfer: .transfer
        case .contractCall: .contractCall
        case let .tokenApproval(approval): try .tokenApproval(Primitives.ApprovalData(approval))
        }
    }
}

extension WcEthereumTransactionData {
    func map() -> WCEthereumTransaction {
        WCEthereumTransaction(
            chainId: chainId,
            from: from,
            to: to,
            value: value,
            gas: gas,
            gasLimit: gasLimit,
            gasPrice: gasPrice,
            maxFeePerGas: maxFeePerGas,
            maxPriorityFeePerGas: maxPriorityFeePerGas,
            nonce: nonce,
            data: data,
        )
    }
}
