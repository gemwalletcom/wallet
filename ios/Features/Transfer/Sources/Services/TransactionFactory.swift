// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

enum TransactionFactory {
    static func makePendingTransaction(
        wallet: Wallet,
        transferData: TransferData,
        transactionData: TransactionData,
        amount: TransferAmount,
        hash: String,
        transactionType: TransactionType,
        simulation: SimulationResult? = nil,
    ) throws -> Primitives.Transaction {
        let senderAddress = try wallet.account(for: transferData.chain).address
        let approval = try transferData.type.approvalData(for: transactionType)

        let recipientAddress: String
        let value: String
        let memo: String
        if let approval {
            recipientAddress = approval.spender
            value = approval.value
            memo = ""
        } else {
            recipientAddress = switch transferData.type {
            case let .swap(_, _, swapData): swapData.data.to
            default: transferData.recipientData.recipient.address
            }
            value = amount.value.description
            memo = transferData.recipientData.recipient.memo ?? ""
        }

        let assetId: AssetId = switch transferData.type {
        case .generic: simulation?.header?.assetId ?? approval.map { AssetId(chain: transferData.chain, tokenId: $0.token) } ?? transferData.type.asset.id
        default: transferData.type.asset.id
        }
        let direction: TransactionDirection = senderAddress == recipientAddress ? .selfTransfer : .outgoing
        let metadata: AnyCodableValue? = switch transferData.type {
        case .swap, .earn: approval == nil ? transferData.type.metadata : .null
        case .transfer, .deposit, .withdrawal, .transferNft, .tokenApprove, .stake, .account, .perpetual, .generic: transferData.type.metadata
        }
        return Transaction(
            id: TransactionId(chain: transferData.chain, hash: hash),
            assetId: assetId,
            from: senderAddress,
            to: recipientAddress,
            contract: nil,
            type: transactionType,
            state: .pending,
            blockNumber: (try? String(transactionData.metadata.getBlockNumber())) ?? "0",
            sequence: (try? String(transactionData.metadata.getSequence())) ?? "0",
            fee: amount.networkFee.description,
            feeAssetId: transactionData.fee.feeAssetId,
            value: value,
            memo: memo,
            direction: direction,
            utxoInputs: [],
            utxoOutputs: [],
            metadata: metadata,
            createdAt: Date(),
        )
    }
}
