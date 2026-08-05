// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.SignableTransaction {
    func map() -> Primitives.SignableTransaction {
        switch self {
        case let .ethereum(data, transactionType): .ethereum(data.map(), transactionType.map())
        case let .solana(data, outputType): .solana(data.transaction, outputType.map())
        case let .sui(data, outputType): .sui(data.transaction, outputType.map())
        case let .ton(data, outputType): .ton(data, outputType.map())
        case let .tron(data, outputType): .tron(data, outputType.map())
        }
    }
}

extension Gemstone.EthereumTransactionData {
    func map() -> Primitives.EthereumTransactionData {
        Primitives.EthereumTransactionData(
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
