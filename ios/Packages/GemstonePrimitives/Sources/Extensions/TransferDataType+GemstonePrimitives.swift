// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemTransactionInputType
import class Gemstone.GemTransferService
import Primitives

private let transferService = GemTransferService()

public extension TransferDataType {
    var inputType: GemTransactionInputType {
        do {
            return try map()
        } catch {
            preconditionFailure("Unencodable transfer data type: \(error)")
        }
    }

    var asset: Asset {
        switch self {
        case let .transfer(asset),
             let .deposit(asset),
             let .withdrawal(asset),
             let .swap(asset, _, _),
             let .stake(asset, _),
             let .account(asset, _),
             let .perpetual(asset, _),
             let .earn(asset, _, _),
             let .tokenApprove(asset, _),
             let .generic(asset, _, _): asset
        case let .transferNft(nftAsset): Asset(nftAsset.chain)
        }
    }

    var feeAsset: Asset {
        do {
            return try Asset(transferService.feeAsset(inputType: inputType))
        } catch {
            preconditionFailure("Undecodable transfer fee asset: \(error)")
        }
    }

    var transactionType: TransactionType {
        do {
            return try TransactionType(transferService.transactionType(inputType: inputType))
        } catch {
            preconditionFailure("Undecodable transaction type: \(error)")
        }
    }

    var assetIds: [AssetId] {
        transferService.assetIds(inputType: inputType).compactMap { try? AssetId(id: $0) }
    }

    var outputType: TransferDataOutputType {
        (try? TransferDataOutputType(transferService.output(inputType: inputType).outputType)) ?? .encodedTransaction
    }

    var outputAction: TransferDataOutputAction {
        (try? TransferDataOutputAction(transferService.output(inputType: inputType).outputAction)) ?? .sign
    }

    func metadata() throws -> AnyCodableValue? {
        try transferService.metadata(inputType: inputType).map { try JSONDecoder().decode(AnyCodableValue.self, from: Data($0.utf8)) }
    }

    func approvalData(for transactionType: TransactionType) throws -> ApprovalData? {
        try transferService.approval(inputType: inputType, transactionType: transactionType.json()).map { try ApprovalData($0) }
    }
}
