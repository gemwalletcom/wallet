// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension GemTransactionLoadMetadata {
    func map() throws -> TransactionLoadMetadata {
        switch self {
        case .none:
            .none
        case let .solana(senderTokenAddress, recipientTokenAddress, tokenProgram, blockHash):
            .solana(
                senderTokenAddress: senderTokenAddress,
                recipientTokenAddress: recipientTokenAddress,
                tokenProgram: tokenProgram?.map(),
                blockHash: blockHash,
            )
        case let .ton(senderTokenAddress, recipientTokenAddress, sequence):
            .ton(
                senderTokenAddress: senderTokenAddress,
                recipientTokenAddress: recipientTokenAddress,
                sequence: sequence,
            )
        case let .cosmos(accountNumber, sequence, chainId):
            .cosmos(accountNumber: UInt64(accountNumber), sequence: sequence, chainId: chainId)
        case let .bitcoin(utxos):
            try .bitcoin(utxos: utxos.map { try $0.map() })
        case let .zcash(utxos, branchId):
            try .zcash(utxos: utxos.map { try $0.map() }, branchId: branchId)
        case let .cardano(utxos):
            try .cardano(utxos: utxos.map { try $0.map() })
        case let .evm(nonce, chainId, contractCall):
            .evm(nonce: UInt64(nonce), chainId: UInt64(chainId), contractCall: contractCall?.map())
        case let .near(sequence, blockHash):
            .near(sequence: sequence, blockHash: blockHash)
        case let .stellar(sequence, isDestinationAddressExist):
            .stellar(sequence: sequence, isDestinationAddressExist: isDestinationAddressExist)
        case let .xrp(sequence, blockNumber):
            .xrp(sequence: sequence, blockNumber: blockNumber)
        case let .algorand(sequence, blockHash, chainId):
            .algorand(sequence: sequence, blockHash: blockHash, chainId: chainId)
        case let .aptos(sequence, data):
            .aptos(sequence: sequence, data: data)
        case let .polkadot(sequence, genesisHash, blockHash, blockNumber, specVersion, transactionVersion, period):
            .polkadot(
                sequence: sequence,
                genesisHash: genesisHash,
                blockHash: blockHash,
                blockNumber: UInt64(blockNumber),
                specVersion: specVersion,
                transactionVersion: transactionVersion,
                period: UInt64(period),
            )
        case let .tron(
            blockNumber,
            blockVersion,
            blockTimestamp,
            transactionTreeRoot,
            parentHash,
            witnessAddress,
            stakeData,
        ):
            .tron(
                blockNumber: UInt64(blockNumber),
                blockVersion: UInt64(blockVersion),
                blockTimestamp: UInt64(blockTimestamp),
                transactionTreeRoot: transactionTreeRoot,
                parentHash: parentHash,
                witnessAddress: witnessAddress,
                stakeData: stakeData.map(),
            )
        case let .sui(messageBytes):
            .sui(messageBytes: messageBytes)
        case let .hyperliquid(order):
            .hyperliquid(order: order?.map())
        }
    }
}

public extension TransactionLoadMetadata {
    func map() -> GemTransactionLoadMetadata {
        switch self {
        case .none:
            .none
        case let .solana(senderTokenAddress, recipientTokenAddress, tokenProgram, blockHash):
            .solana(
                senderTokenAddress: senderTokenAddress,
                recipientTokenAddress: recipientTokenAddress,
                tokenProgram: tokenProgram?.map(),
                blockHash: blockHash,
            )
        case let .ton(senderTokenAddress, recipientTokenAddress, sequence):
            .ton(senderTokenAddress: senderTokenAddress, recipientTokenAddress: recipientTokenAddress, sequence: sequence)
        case let .cosmos(accountNumber, sequence, chainId):
            .cosmos(accountNumber: UInt64(accountNumber), sequence: sequence, chainId: chainId)
        case let .bitcoin(utxos):
            .bitcoin(utxos: utxos.map { $0.map() })
        case let .zcash(utxos, branchId):
            .zcash(utxos: utxos.map { $0.map() }, branchId: branchId)
        case let .cardano(utxos):
            .cardano(utxos: utxos.map { $0.map() })
        case let .evm(nonce, chainId, contractCall):
            .evm(
                nonce: UInt64(nonce),
                chainId: UInt64(chainId),
                contractCall: contractCall?.map(),
            )
        case let .near(sequence, blockHash):
            .near(sequence: sequence, blockHash: blockHash)
        case let .stellar(sequence, isDestinationAddressExist):
            .stellar(sequence: sequence, isDestinationAddressExist: isDestinationAddressExist)
        case let .xrp(sequence, blockNumber):
            .xrp(sequence: sequence, blockNumber: blockNumber)
        case let .algorand(sequence, blockHash, chainId):
            .algorand(sequence: sequence, blockHash: blockHash, chainId: chainId)
        case let .aptos(sequence, data):
            .aptos(sequence: sequence, data: data)
        case let .polkadot(sequence, genesisHash, blockHash, blockNumber, specVersion, transactionVersion, period):
            .polkadot(
                sequence: sequence,
                genesisHash: genesisHash,
                blockHash: blockHash,
                blockNumber: UInt64(blockNumber),
                specVersion: specVersion,
                transactionVersion: transactionVersion,
                period: UInt64(period),
            )
        case let .tron(
            blockNumber,
            blockVersion,
            blockTimestamp,
            transactionTreeRoot,
            parentHash,
            witnessAddress,
            stakeData,
        ):
            .tron(
                blockNumber: UInt64(blockNumber),
                blockVersion: UInt64(blockVersion),
                blockTimestamp: UInt64(blockTimestamp),
                transactionTreeRoot: transactionTreeRoot,
                parentHash: parentHash,
                witnessAddress: witnessAddress,
                stakeData: stakeData.map(),
            )
        case let .sui(messageBytes):
            .sui(messageBytes: messageBytes)
        case let .hyperliquid(order):
            .hyperliquid(order: order?.map())
        }
    }
}

extension Gemstone.TronStakeData {
    func map() -> Primitives.TronStakeData {
        switch self {
        case let .votes(votes): .votes(votes.map { $0.map() })
        case let .unfreeze(amounts): .unfreeze(amounts.map { $0.map() })
        }
    }
}

extension Primitives.TronStakeData {
    func map() -> Gemstone.TronStakeData {
        switch self {
        case let .votes(votes): .votes(votes.map { $0.map() })
        case let .unfreeze(amounts): .unfreeze(amounts.map { $0.map() })
        }
    }
}

extension Gemstone.TronVote {
    func map() -> Primitives.TronVote {
        Primitives.TronVote(validator: validator, count: count)
    }
}

extension Primitives.TronVote {
    func map() -> Gemstone.TronVote {
        Gemstone.TronVote(validator: validator, count: count)
    }
}

extension Gemstone.TronUnfreeze {
    func map() -> Primitives.TronUnfreeze {
        Primitives.TronUnfreeze(resource: resource.map(), amount: amount)
    }
}

extension Primitives.TronUnfreeze {
    func map() -> Gemstone.TronUnfreeze {
        Gemstone.TronUnfreeze(resource: resource.map(), amount: amount)
    }
}

extension GemHyperliquidOrder {
    func map() -> HyperliquidOrder {
        HyperliquidOrder(
            approveAgentRequired: approveAgentRequired,
            approveReferralRequired: approveReferralRequired,
            approveBuilderRequired: approveBuilderRequired,
            builderFeeBps: UInt32(builderFeeBps),
            agentAddress: agentAddress,
            agentPrivateKey: agentPrivateKey,
        )
    }
}

extension HyperliquidOrder {
    func map() -> GemHyperliquidOrder {
        GemHyperliquidOrder(
            approveAgentRequired: approveAgentRequired,
            approveReferralRequired: approveReferralRequired,
            approveBuilderRequired: approveBuilderRequired,
            builderFeeBps: builderFeeBps,
            agentAddress: agentAddress,
            agentPrivateKey: agentPrivateKey,
        )
    }
}
