// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemTransactionInputType
import class Gemstone.GemTransferService
import Primitives

public extension TransferDataType {
    var inputType: GemTransactionInputType {
        try! map()
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

    func feeAsset(transferService: GemTransferService) -> Asset {
        transferService.feeAsset(inputType: inputType).map()
    }

    func transactionType(transferService: GemTransferService) -> TransactionType {
        TransactionType(core: transferService.transactionType(inputType: inputType))
    }

    func assetIds(transferService: GemTransferService) -> [AssetId] {
        transferService.assetIds(inputType: inputType).map { try! AssetId(id: $0) }
    }

    func outputType(transferService: GemTransferService) -> TransferDataOutputType {
        TransferDataOutputType(core: transferService.output(inputType: inputType).outputType)
    }

    func outputAction(transferService: GemTransferService) -> TransferDataOutputAction {
        TransferDataOutputAction(core: transferService.output(inputType: inputType).outputAction)
    }

    func metadata(transferService: GemTransferService) throws -> AnyCodableValue? {
        try transferService.metadata(inputType: inputType).map { try JSONDecoder().decode(AnyCodableValue.self, from: Data($0.utf8)) }
    }

    func approvalData(for transactionType: TransactionType, transferService: GemTransferService) throws -> ApprovalData? {
        try transferService.approval(inputType: inputType, transactionType: transactionType.json()).map { try ApprovalData($0) }
    }
}
