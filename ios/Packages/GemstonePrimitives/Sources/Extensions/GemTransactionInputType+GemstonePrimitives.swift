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
            try TransferDataType.transfer(asset.map())
        case let .deposit(asset):
            try TransferDataType.deposit(asset.map())
        case let .swap(fromAsset, toAsset, gemSwapData):
            try TransferDataType.swap(fromAsset.map(), toAsset.map(), Primitives.SwapData(gemSwapData))
        case let .transferNft(_, nftAsset):
            try TransferDataType.transferNft(Primitives.NFTAsset(nftAsset))
        case let .stake(asset, type):
            try TransferDataType.stake(asset.map(), Primitives.StakeType(type))
        case let .tokenApprove(asset, approvalData):
            try TransferDataType.tokenApprove(asset.map(), Primitives.ApprovalData(approvalData))
        case let .generic(asset, metadata, extra):
            try TransferDataType.generic(asset: asset.map(), metadata: Primitives.ApplicationMetadata(metadata), extra: extra.map())
        case let .account(asset, accountType):
            try TransferDataType.account(asset.map(), Primitives.AccountDataType(accountType))
        case let .perpetual(asset: asset, perpetualType: perpetualType):
            try TransferDataType.perpetual(asset.map(), Primitives.PerpetualType(perpetualType))
        case let .earn(asset, earnType, data):
            try TransferDataType.earn(asset.map(), Primitives.EarnType(earnType), Primitives.ContractCallData(data))
        case let .withdrawal(asset):
            try TransferDataType.withdrawal(asset.map())
        }
    }
}

public extension TransferDataType {
    func map() throws -> GemTransactionInputType {
        switch self {
        case let .transfer(asset):
            return .transfer(asset: asset.map())
        case let .deposit(asset):
            return .deposit(asset: asset.map())
        case let .swap(fromAsset, toAsset, swapData):
            return try .swap(fromAsset: fromAsset.map(), toAsset: toAsset.map(), swapData: swapData.json())
        case let .transferNft(nftAsset):
            return try .transferNft(asset: Primitives.Asset(nftAsset.chain).map(), nftAsset: nftAsset.json())
        case let .stake(asset, stakeType):
            return try .stake(asset: asset.map(), stakeType: stakeType.json())
        case let .tokenApprove(asset, approvalData):
            return try .tokenApprove(asset: asset.map(), approvalData: approvalData.json())
        case let .generic(asset, metadata, extra):
            return try .generic(asset: asset.map(), metadata: metadata.json(), extra: extra.map())
        case let .withdrawal(asset):
            return .withdrawal(asset: asset.map())
        case let .account(asset, accountData):
            return try .account(asset: asset.map(), accountType: accountData.json())
        case let .perpetual(asset, perpetualType):
            return try .perpetual(asset: asset.map(), perpetualType: perpetualType.json())
        case let .earn(asset, earnType, data):
            return try .earn(asset: asset.map(), earnType: earnType.json(), data: data.json())
        }
    }
}
