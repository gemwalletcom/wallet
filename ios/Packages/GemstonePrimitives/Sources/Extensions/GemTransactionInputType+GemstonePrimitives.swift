// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension GemTransactionInputType {
    func getAsset() -> Gemstone.Asset {
        switch self {
        case let .transfer(asset): asset
        case let .deposit(asset): asset
        case let .transferNft(asset, _): asset
        case let .swap(fromAsset, _, _): fromAsset
        case let .stake(asset, _): asset
        case let .tokenApprove(asset, _): asset
        case let .generic(asset, _, _): asset
        case let .account(asset, _): asset
        case .perpetual(asset: let asset, perpetualType: _): asset
        case .earn(asset: let asset, earnType: _, data: _): asset
        case let .withdrawal(asset): asset
        }
    }
}

public extension GemTransactionInputType {
    func map() throws -> TransferDataType {
        switch self {
        case let .transfer(asset):
            try TransferDataType.transfer(Primitives.Asset(asset))
        case let .deposit(asset):
            try TransferDataType.deposit(Primitives.Asset(asset))
        case let .swap(fromAsset, toAsset, gemSwapData):
            try TransferDataType.swap(Primitives.Asset(fromAsset), Primitives.Asset(toAsset), Primitives.SwapData(gemSwapData))
        case let .transferNft(_, nftAsset):
            try TransferDataType.transferNft(Primitives.NFTAsset(nftAsset))
        case let .stake(asset, type):
            try TransferDataType.stake(Primitives.Asset(asset), Primitives.StakeType(type))
        case let .tokenApprove(asset, approvalData):
            try TransferDataType.tokenApprove(Primitives.Asset(asset), Primitives.ApprovalData(approvalData))
        case let .generic(asset, metadata, extra):
            try TransferDataType.generic(asset: Primitives.Asset(asset), metadata: Primitives.ApplicationMetadata(metadata), extra: extra.map())
        case let .account(asset, accountType):
            try TransferDataType.account(Primitives.Asset(asset), Primitives.AccountDataType(accountType))
        case let .perpetual(asset: asset, perpetualType: perpetualType):
            try TransferDataType.perpetual(Primitives.Asset(asset), Primitives.PerpetualType(perpetualType))
        case let .earn(asset, earnType, data):
            try TransferDataType.earn(Primitives.Asset(asset), Primitives.EarnType(earnType), Primitives.ContractCallData(data))
        case let .withdrawal(asset):
            try TransferDataType.withdrawal(Primitives.Asset(asset))
        }
    }
}

public extension TransferDataType {
    func map() throws -> GemTransactionInputType {
        switch self {
        case let .transfer(asset):
            return .transfer(asset: asset.json())
        case let .deposit(asset):
            return .deposit(asset: asset.json())
        case let .swap(fromAsset, toAsset, swapData):
            return try .swap(fromAsset: fromAsset.json(), toAsset: toAsset.json(), swapData: swapData.json())
        case let .transferNft(nftAsset):
            return try .transferNft(asset: Primitives.Asset(nftAsset.chain).json(), nftAsset: nftAsset.json())
        case let .stake(asset, stakeType):
            return try .stake(asset: asset.json(), stakeType: stakeType.json())
        case let .tokenApprove(asset, approvalData):
            return try .tokenApprove(asset: asset.json(), approvalData: approvalData.json())
        case let .generic(asset, metadata, extra):
            return try .generic(asset: asset.json(), metadata: metadata.json(), extra: extra.map())
        case let .withdrawal(asset):
            return .withdrawal(asset: asset.json())
        case let .account(asset, accountData):
            return try .account(asset: asset.json(), accountType: accountData.json())
        case let .perpetual(asset, perpetualType):
            return try .perpetual(asset: asset.json(), perpetualType: perpetualType.json())
        case let .earn(asset, earnType, data):
            return try .earn(asset: asset.json(), earnType: earnType.json(), data: data.json())
        }
    }
}
