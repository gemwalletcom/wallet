// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemTransactionInputType
import class Gemstone.GemTransferService
import Primitives

public extension TransferDataType {
    var inputType: GemTransactionInputType {
        do {
            return try map()
        } catch {
            preconditionFailure("Unencodable transfer data type: \(error)")
        }
    }

    var asset: Asset {
        do {
            return try Asset(GemTransferService().asset(inputType: inputType))
        } catch {
            preconditionFailure("Undecodable transfer asset: \(error)")
        }
    }

    var feeAsset: Asset {
        do {
            return try Asset(GemTransferService().feeAsset(inputType: inputType))
        } catch {
            preconditionFailure("Undecodable transfer fee asset: \(error)")
        }
    }

    var transactionType: TransactionType {
        do {
            return try TransactionType(GemTransferService().transactionType(inputType: inputType))
        } catch {
            preconditionFailure("Undecodable transaction type: \(error)")
        }
    }

    var assetIds: [AssetId] {
        GemTransferService().assetIds(inputType: inputType).compactMap { try? AssetId(id: $0) }
    }

    var outputType: TransferDataOutputType {
        (try? TransferDataOutputType(GemTransferService().output(inputType: inputType).outputType)) ?? .encodedTransaction
    }

    var outputAction: TransferDataOutputAction {
        (try? TransferDataOutputAction(GemTransferService().output(inputType: inputType).outputAction)) ?? .sign
    }

    func metadata() throws -> AnyCodableValue? {
        try GemTransferService().metadata(inputType: inputType).map { try JSONDecoder().decode(AnyCodableValue.self, from: Data($0.utf8)) }
    }

    func approvalData(for transactionType: TransactionType) throws -> ApprovalData? {
        try GemTransferService().approval(inputType: inputType, transactionType: transactionType.json()).map { try ApprovalData($0) }
    }
}
