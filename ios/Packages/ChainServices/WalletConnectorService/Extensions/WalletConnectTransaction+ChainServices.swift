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
    func map() -> WalletConnectorTransaction {
        switch self {
        case let .ethereum(data, kind): .ethereum(data.map(), kind.map())
        case let .solana(data, outputType): .solana(data.transaction, outputType.map())
        case let .sui(data, outputType): .sui(data.transaction, outputType.map())
        case let .ton(data, outputType): .ton(data, outputType.map())
        case let .tron(data, outputType): .tron(data, outputType.map())
        }
    }
}

extension EvmTransactionKind {
    func map() -> WalletConnectorEVMTransactionKind {
        switch self {
        case .transfer: .transfer
        case .contractCall: .contractCall
        case let .tokenApproval(approval): .tokenApproval(approval.map())
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
