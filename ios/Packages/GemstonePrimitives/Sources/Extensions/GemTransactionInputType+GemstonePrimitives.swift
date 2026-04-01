// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension GemTransactionInputType {
    func getAsset() -> GemAsset {
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
            try TransferDataType.swap(fromAsset.map(), toAsset.map(), gemSwapData.map())
        case let .transferNft(_, nftAsset):
            try TransferDataType.transferNft(nftAsset.map())
        case let .stake(asset, type):
            try TransferDataType.stake(asset.map(), type.map())
        case let .tokenApprove(asset, approvalData):
            try TransferDataType.tokenApprove(asset.map(), approvalData.map())
        case let .generic(asset, metadata, extra):
            try TransferDataType.generic(asset: asset.map(), metadata: metadata.map(), extra: extra.map())
        case let .account(asset, accountType):
            try TransferDataType.account(asset.map(), accountType.map())
        case let .perpetual(asset: asset, perpetualType: perpetualType):
            try TransferDataType.perpetual(asset.map(), perpetualType.map())
        case let .earn(asset, earnType, data):
            try TransferDataType.earn(asset.map(), earnType.map(), data.map())
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
            return try .swap(fromAsset: fromAsset.map(), toAsset: toAsset.map(), swapData: swapData.map())
        case let .transferNft(nftAsset):
            return .transferNft(asset: Asset(nftAsset.chain).map(), nftAsset: nftAsset.map())
        case let .stake(asset, stakeType):
            return .stake(asset: asset.map(), stakeType: stakeType.map())
        case let .tokenApprove(asset, approvalData):
            return .tokenApprove(asset: asset.map(), approvalData: approvalData.map())
        case let .generic(asset, metadata, extra):
            return .generic(asset: asset.map(), metadata: metadata.map(), extra: extra.map())
        case let .withdrawal(asset):
            if asset.chain == .hyperCore {
                return .transfer(asset: asset.map())
            }
            throw AnyError("Unsupported transaction type: \(self)")
        case let .account(asset, accountData):
            return .account(asset: asset.map(), accountType: accountData.map())
        case let .perpetual(asset, perpetualType):
            return .perpetual(asset: asset.map(), perpetualType: perpetualType.map())
        case let .earn(asset, earnType, data):
            return .earn(asset: asset.map(), earnType: earnType.map(), data: data.map())
        }
    }
}
