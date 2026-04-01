// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum TransferDataType: Hashable, Equatable, Sendable {
    case transfer(Asset)
    case deposit(Asset)
    case withdrawal(Asset)
    case transferNft(NFTAsset)
    case swap(Asset, Asset, SwapData)
    case tokenApprove(Asset, ApprovalData)
    case stake(Asset, StakeType)
    case account(Asset, AccountDataType)
    case perpetual(Asset, PerpetualType)
    case earn(Asset, EarnType, ContractCallData)
    case generic(asset: Asset, metadata: WalletConnectionSessionAppMetadata, extra: TransferDataExtra)

    public var transactionType: TransactionType {
        switch self {
        case .transfer: .transfer
        case .deposit: .transfer
        case .withdrawal: .transfer
        case .generic: .smartContractCall
        case .transferNft: .transferNFT
        case .tokenApprove: .tokenApproval
        case .swap: .swap
        case let .stake(_, type):
            switch type {
            case .stake: .stakeDelegate
            case .unstake: .stakeUndelegate
            case .redelegate: .stakeRedelegate
            case .rewards: .stakeRewards
            case .withdraw: .stakeWithdraw
            case .freeze: .stakeFreeze
            case .unfreeze: .stakeUnfreeze
            }
        case .account: .assetActivation
        case let .earn(_, type, _):
            switch type {
            case .deposit: .earnDeposit
            case .withdraw: .earnWithdraw
            }
        case let .perpetual(_, type):
            switch type {
            case .open, .increase: .perpetualOpenPosition
            case .close, .reduce: .perpetualClosePosition
            case .modify: .perpetualModifyPosition
            }
        }
    }

    public var chain: Chain {
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
             let .generic(asset, _, _): asset.chain
        case let .transferNft(asset): asset.chain
        }
    }

    public var metadata: AnyCodableValue? {
        switch self {
        case let .swap(fromAsset, toAsset, data):
            return .encode(TransactionSwapMetadata(
                fromAsset: fromAsset.id,
                fromValue: data.quote.fromValue,
                toAsset: toAsset.id,
                toValue: data.quote.toValue,
                provider: data.quote.providerData.provider.rawValue,
            ))
        case let .transferNft(asset):
            return .encode(TransactionNFTTransferMetadata(assetId: asset.id, name: asset.name))
        case let .perpetual(_, type):
            guard let direction = type.data?.direction else { return nil }
            return .encode(
                TransactionPerpetualMetadata(pnl: 0, price: 0, direction: direction, isLiquidation: .none, provider: nil),
            )
        case let .stake(_, type):
            switch type {
            case let .freeze(resource), let .unfreeze(resource):
                return .encode(TransactionResourceTypeMetadata(resourceType: resource))
            case .stake, .unstake, .redelegate, .rewards, .withdraw:
                return nil
            }
        case let .generic(_, _, extra):
            return .encode(TransactionWalletConnectMetadata(outputAction: extra.outputAction))
        case .transfer,
             .deposit,
             .withdrawal,
             .tokenApprove,
             .account,
             .earn:
            return nil
        }
    }

    public var assetIds: [AssetId] {
        switch self {
        case let .transfer(asset),
             let .deposit(asset),
             let .withdrawal(asset),
             let .tokenApprove(asset, _),
             let .stake(asset, _),
             let .generic(asset, _, _),
             let .account(asset, _),
             let .perpetual(asset, _),
             let .earn(asset, _, _): [asset.id]
        case let .swap(from, to, _): [from.id, to.id]
        case .transferNft: []
        }
    }

    public var outputType: TransferDataOutputType {
        switch self {
        case let .generic(_, _, extra): extra.outputType
        default: .encodedTransaction
        }
    }

    public var outputAction: TransferDataOutputAction {
        switch self {
        case let .generic(_, _, extra): extra.outputAction
        default: .send
        }
    }

    public func swap() throws -> (Asset, Asset, data: SwapData) {
        guard case let .swap(fromAsset, toAsset, data) = self else {
            throw AnyError("SwapQuoteData missed")
        }
        return (fromAsset, toAsset, data)
    }

    public func earn() throws -> (Asset, EarnType, data: ContractCallData) {
        guard case let .earn(asset, earnType, data) = self else {
            throw AnyError("EarnData missed")
        }
        return (asset, earnType, data)
    }

    public var shouldIgnoreValueCheck: Bool {
        switch self {
        case .transferNft, .stake, .account, .tokenApprove, .perpetual, .earn: true
        case .transfer, .deposit, .withdrawal, .swap, .generic: false
        }
    }

    public func withGasLimit(_ gasLimit: String) -> TransferDataType {
        guard case let .swap(from, to, swapData) = self else { return self }
        return .swap(from, to, swapData.withGasLimit(gasLimit))
    }

    public var recentActivityData: RecentActivityData? {
        switch self {
        case let .transfer(asset): RecentActivityData(type: .transfer, assetId: asset.id, toAssetId: nil)
        case let .swap(from, to, _): RecentActivityData(type: .swap, assetId: from.id, toAssetId: to.id)
        default: nil
        }
    }
}
