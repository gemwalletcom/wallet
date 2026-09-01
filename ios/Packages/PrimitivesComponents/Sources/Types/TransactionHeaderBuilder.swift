// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemTransactionInputType
import GemstonePrimitives
import struct Gemstone.GemConfirmMetadata
import BigInt
import Foundation
import Primitives

public enum TransactionHeaderTypeBuilder {
    public static func build(
        infoModel: TransactionInfoViewModel,
        transaction: Transaction,
        metadata: TransactionExtendedMetadata?,
    ) -> TransactionHeaderType {
        let inputType: TransactionHeaderInputType = {
            switch transaction.type {
            case .transfer,
                 .stakeDelegate,
                 .stakeUndelegate,
                 .stakeRedelegate,
                 .stakeRewards,
                 .stakeWithdraw,
                 .smartContractCall,
                 .stakeFreeze,
                 .stakeUnfreeze:
                return .amount(showFiat: true)
            case .swap:
                guard let metadata, let input = SwapMetadataViewModel(metadata: metadata).headerInput else {
                    return .amount(showFiat: true)
                }
                return .swap(input)
            case .assetActivation:
                return .symbol
            case .tokenApproval:
                return .assetImage
            case .transferNFT:
                guard let metadata = transaction.metadata?.decode(TransactionNFTTransferMetadata.self) else {
                    return .amount(showFiat: false)
                }
                return .nft(name: metadata.name, id: metadata.assetId.identifier)
            case .perpetualOpenPosition, .perpetualClosePosition, .perpetualModifyPosition:
                return .symbol
            case .earnDeposit, .earnWithdraw:
                return .amount(showFiat: true)
            }
        }()
        return infoModel.headerType(input: inputType)
    }

    public static func build(
        infoModel: TransactionInfoViewModel,
        dataType: GemTransactionInputType,
        metadata: GemConfirmMetadata?,
    ) -> TransactionHeaderType {
        let inputType: TransactionHeaderInputType = {
            switch dataType {
            case .transfer,
                 .deposit,
                 .withdrawal,
                 .stake,
                 .generic:
                return .amount(showFiat: true)
            case .tokenApprove:
                return .assetImage
            case let .transferNft(_, nftAsset):
                let nft = Primitives.NFTAsset(core: nftAsset)
                return .nft(name: nft.name, id: nft.id.identifier)
            case let .account(_, type):
                switch Primitives.AccountDataType(core: type) {
                case .activate:
                    return .amount(showFiat: false)
                }
            case let .swap(fromAsset, toAsset, data):
                let assetPrices = (metadata?.assetPrices ?? [:]).map { assetId, price in
                    price.mapToAssetPrice(assetId: assetId)
                }

                let from = fromAsset.map()
                let to = toAsset.map()
                let quote = Primitives.SwapData(core: data).quote
                let model = SwapMetadataViewModel(
                    metadata: TransactionExtendedMetadata(
                        assets: [from, to],
                        assetPrices: assetPrices,
                        metadata: .encode(TransactionSwapMetadata(
                            fromAsset: from.id,
                            fromValue: quote.fromValue,
                            toAsset: to.id,
                            toValue: quote.toValue,
                            provider: quote.providerData.provider.rawValue,
                        )),
                    ),
                )

                guard let input = model.headerInput else {
                    return .amount(showFiat: true)
                }
                return .swap(input)
            case .perpetual:
                return .symbol
            case .earn:
                return .amount(showFiat: true)
            }
        }()
        return infoModel.headerType(input: inputType)
    }
}
