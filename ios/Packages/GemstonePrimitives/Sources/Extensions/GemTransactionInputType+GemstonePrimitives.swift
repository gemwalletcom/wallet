// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemTransactionInputType {
    var asset: Primitives.Asset {
        switch self {
        case let .transfer(asset),
             let .deposit(asset),
             let .withdrawal(asset),
             let .stake(asset, _),
             let .tokenApprove(asset, _),
             let .account(asset, _),
             let .perpetual(asset, _),
             let .transferNft(asset, _),
             let .generic(asset, _, _): asset.map()
        case let .earn(asset, _, _): asset.map()
        case let .swap(fromAsset, _, _): fromAsset.map()
        }
    }

    var chain: Primitives.Chain {
        asset.chain
    }

    var applicationMetadata: Primitives.ApplicationMetadata? {
        guard case let .generic(_, metadata, _) = self else { return nil }
        return Primitives.ApplicationMetadata(core: metadata)
    }
}

public extension GemTransactionInputType {
    func feeAsset(transferService: GemTransferService) -> Primitives.Asset {
        transferService.feeAsset(inputType: self).map()
    }

    func transactionType(transferService: GemTransferService) -> Primitives.TransactionType {
        Primitives.TransactionType(core: transferService.transactionType(inputType: self))
    }

    func assetIds(transferService: GemTransferService) -> [Primitives.AssetId] {
        transferService.assetIds(inputType: self).map { Primitives.AssetId(core: $0) }
    }

    func outputType(transferService: GemTransferService) -> Primitives.TransferDataOutputType {
        Primitives.TransferDataOutputType(core: transferService.output(inputType: self).outputType)
    }

    func outputAction(transferService: GemTransferService) -> Primitives.TransferDataOutputAction {
        Primitives.TransferDataOutputAction(core: transferService.output(inputType: self).outputAction)
    }

    func metadata(transferService: GemTransferService) throws -> AnyCodableValue? {
        try transferService.metadata(inputType: self).map { try JSONDecoder().decode(AnyCodableValue.self, from: Data($0.utf8)) }
    }

    func approvalData(for transactionType: Primitives.TransactionType, transferService: GemTransferService) throws -> Primitives.ApprovalData? {
        try transferService.approval(inputType: self, transactionType: transactionType.json()).map { try Primitives.ApprovalData($0) }
    }
}

public extension GemTransactionInputType {
    static func transfer(_ asset: Primitives.Asset) -> Self {
        .transfer(asset: asset.map())
    }

    static func deposit(_ asset: Primitives.Asset) -> Self {
        .deposit(asset: asset.map())
    }

    static func withdrawal(_ asset: Primitives.Asset) -> Self {
        .withdrawal(asset: asset.map())
    }

    static func transferNft(_ nftAsset: Primitives.NFTAsset) -> Self {
        .transferNft(asset: Primitives.Asset(nftAsset.chain).map(), nftAsset: nftAsset.json())
    }

    static func swap(_ fromAsset: Primitives.Asset, _ toAsset: Primitives.Asset, _ swapData: Primitives.SwapData) -> Self {
        .swap(fromAsset: fromAsset.map(), toAsset: toAsset.map(), swapData: swapData.json())
    }

    static func tokenApprove(_ asset: Primitives.Asset, _ approvalData: Primitives.ApprovalData) -> Self {
        .tokenApprove(asset: asset.map(), approvalData: approvalData.json())
    }

    static func stake(_ asset: Primitives.Asset, _ stakeType: Primitives.StakeType) -> Self {
        .stake(asset: asset.map(), stakeType: stakeType.json())
    }

    static func account(_ asset: Primitives.Asset, _ accountType: Primitives.AccountDataType) -> Self {
        .account(asset: asset.map(), accountType: accountType.json())
    }

    static func perpetual(_ asset: Primitives.Asset, _ perpetualType: Primitives.PerpetualType) -> Self {
        .perpetual(asset: asset.map(), perpetualType: perpetualType.json())
    }

    static func earn(_ asset: Primitives.Asset, _ earnType: Primitives.EarnType, _ data: Primitives.ContractCallData) -> Self {
        .earn(asset: asset.map(), earnType: earnType.json(), data: data.json())
    }

    static func generic(asset: Primitives.Asset, metadata: Primitives.ApplicationMetadata, extra: Primitives.TransferDataExtra) -> Self {
        .generic(asset: asset.map(), metadata: metadata.json(), extra: extra.map())
    }
}
