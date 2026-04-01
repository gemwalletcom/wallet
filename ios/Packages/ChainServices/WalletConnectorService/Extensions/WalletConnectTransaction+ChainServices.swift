// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

import enum Gemstone.WalletConnectTransaction
import struct Gemstone.WcEthereumTransactionData
import struct Gemstone.WcSolanaTransactionData
import struct Gemstone.WcSuiTransactionData

extension WalletConnectTransaction {
    func map() -> WalletConnectorTransaction {
        switch self {
        case let .ethereum(data): .ethereum(data.map())
        case let .solana(data, outputType): .solana(data.transaction, outputType.map())
        case let .sui(data, outputType): .sui(data.transaction, outputType.map())
        case let .ton(messages, outputType): .ton(messages, outputType.map())
        case let .bitcoin(data, outputType): .bitcoin(data, outputType.map())
        case let .tron(data, outputType): .tron(data, outputType.map())
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
