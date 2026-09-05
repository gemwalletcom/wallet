// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemTransactionHeaderKind
import enum Gemstone.GemTransactionInputType
import GemstonePrimitives
import struct Gemstone.GemConfirmMetadata
import BigInt
import Foundation
import Primitives

public enum TransactionHeaderTypeBuilder {
    public static func build(
        infoModel: TransactionInfoViewModel,
        kind: GemTransactionHeaderKind,
        transaction: Transaction,
        metadata: TransactionExtendedMetadata?,
    ) -> TransactionHeaderType {
        let inputType: TransactionHeaderInputType = {
            switch kind {
            case let .amount(showsFiat):
                return .amount(showFiat: showsFiat)
            case .swap:
                guard let metadata, let input = SwapMetadataViewModel(metadata: metadata).headerInput else {
                    return .amount(showFiat: true)
                }
                return .swap(input)
            case .nft:
                guard let metadata = transaction.metadata?.decode(TransactionNFTTransferMetadata.self) else {
                    return .amount(showFiat: false)
                }
                return .nft(name: metadata.name, id: metadata.assetId.identifier)
            case .symbol:
                return .symbol
            case .assetImage:
                return .assetImage
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
            switch dataType.headerKind() {
            case let .amount(showsFiat):
                return .amount(showFiat: showsFiat)
            case .nft:
                guard case let .transferNft(_, nftAsset) = dataType else { return .amount(showFiat: false) }
                let nft = nftAsset.map()
                return .nft(name: nft.name, id: nft.id.identifier)
            case .swap:
                guard case let .swap(fromAsset, toAsset, data) = dataType else { return .amount(showFiat: true) }
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
            case .symbol:
                return .symbol
            case .assetImage:
                return .assetImage
            }
        }()
        return infoModel.headerType(input: inputType)
    }
}
