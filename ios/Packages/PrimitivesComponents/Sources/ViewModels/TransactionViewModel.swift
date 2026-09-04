// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.Resource
import BigInt
import Components
import Formatters
import Foundation
import enum Gemstone.GemAmountSign
import enum Gemstone.GemImage
import class Gemstone.GemTransactionSummary
import enum Gemstone.GemTransactionValue
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct TransactionViewModel: Sendable {
    public let transaction: TransactionExtended

    private let row: GemTransactionSummary
    private let currency: String
    private let formatter: ValueFormatter = .short

    public init(
        transaction: TransactionExtended,
        currency: String,
    ) {
        row = GemTransactionSummary(transaction: transaction.transaction.json())
        self.transaction = transaction
        self.currency = currency
    }

    public var assetImage: AssetImage {
        let asset = AssetIdViewModel(assetId: assetId).assetImage
        if let nftMetadata = transaction.transaction.metadata?.decode(TransactionNFTTransferMetadata.self) {
            return AssetImage(
                type: .text(""),
                imageURL: GemImage.nftAsset(assetId: nftMetadata.assetId.identifier).imageURL,
                placeholder: asset.placeholder,
                chainPlaceholder: overlayImage,
            )
        }
        return AssetImage(
            type: asset.type,
            imageURL: asset.imageURL,
            placeholder: asset.placeholder,
            chainPlaceholder: overlayImage,
        )
    }

    public var overlayImage: Image? {
        switch transaction.transaction.type {
        case .transfer, .transferNFT, .smartContractCall:
            switch transaction.transaction.direction {
            case .incoming: Images.Transaction.incoming
            case .outgoing, .selfTransfer: Images.Transaction.outgoing
            }
        case .swap,
             .tokenApproval,
             .stakeDelegate,
             .stakeUndelegate,
             .stakeRewards,
             .stakeRedelegate,
             .stakeWithdraw,
             .assetActivation,
             .perpetualOpenPosition,
             .perpetualClosePosition,
             .stakeFreeze,
             .stakeUnfreeze,
             .perpetualModifyPosition,
             .earnDeposit,
             .earnWithdraw: AssetIdViewModel(assetId: assetId).assetImage.chainPlaceholder
        }
    }

    public var infoModel: TransactionInfoViewModel {
        infoModel(sign: amountSign)
    }

    private var amountSign: GemAmountSign {
        guard case let .amount(sign) = row.value() else { return .none }
        return sign
    }

    private func infoModel(sign: GemAmountSign) -> TransactionInfoViewModel {
        TransactionInfoViewModel(
            currency: currency,
            asset: transaction.asset,
            assetPrice: transaction.price,
            feeAsset: transaction.feeAsset,
            feeAssetPrice: transaction.feePrice,
            value: transaction.transaction.valueBigInt,
            feeValue: transaction.transaction.feeBigInt,
            direction: sign.direction,
        )
    }

    public var titleTextValue: TextValue {
        TextValue(
            text: row.title().title,
            style: TextStyle(font: Font.system(.body, weight: .medium), color: .primary),
        )
    }

    public var titleTagType: TitleTagType {
        TransactionStateViewModel(state: transaction.transaction.state).showsProgress ? .progressView() : .none
    }

    public var titleTagTextValue: TextValue? {
        let model = TransactionStateViewModel(state: transaction.transaction.state)
        let title: String? = switch transaction.transaction.state {
        case .confirmed: .none
        case .pending, .inTransit, .failed, .reverted, .refunded: model.title
        }
        return title.map {
            TextValue(
                text: $0,
                style: TextStyle(
                    font: Font.system(.footnote, weight: .medium),
                    color: model.color,
                    background: model.background,
                ),
            )
        }
    }

    public var titleExtraTextValue: TextValue? {
        let title: String? = switch row.subtitle() {
        case let .toAddress(address): participantTitle(prefix: Localized.Transfer.to, address: address, chain: assetId.chain)
        case let .fromAddress(address): participantTitle(prefix: Localized.Transfer.from, address: address, chain: assetId.chain)
        case let .toResource(resource): resourceTitle(prefix: Localized.Transfer.to, resource: resource)
        case let .fromResource(resource): resourceTitle(prefix: Localized.Transfer.from, resource: resource)
        case let .price(value):
            String(format: "%@: %@", Localized.Asset.price, AmountDisplay.currency(value: value, currencyCode: Currency.usd.rawValue, showSign: false).text)
        case .none: .none
        }

        return title.map {
            TextValue(
                text: $0,
                style: .footnote,
            )
        }
    }

    public var subtitleTextValue: TextValue? {
        amountTextValue(row.value(), textStyle: nil)
    }

    public var subtitleExtraTextValue: TextValue? {
        amountTextValue(row.equivalentValue(), textStyle: .footnote)
    }

    private func amountTextValue(_ value: GemTransactionValue, textStyle: TextStyle?) -> TextValue? {
        switch value {
        case .none:
            return .none
        case .assetSymbol:
            return AmountDisplay.symbol(asset: transaction.asset).amount
        case let .amount(sign):
            return infoModel(sign: sign).amountDisplay(formatter: formatter).amount
        case .swapReceived:
            return swapAmount(assetId: swapMetadata?.toAsset, value: swapMetadata?.toValue, sign: .incoming, textStyle: textStyle)
        case .swapSpent:
            return swapAmount(assetId: swapMetadata?.fromAsset, value: swapMetadata?.fromValue, sign: .outgoing, textStyle: textStyle)
        case .perpetualNotional:
            return AmountDisplay.numeric(
                asset: Chain.hyperCore.defaultAsset(type: .perpetual),
                price: Price(price: 1, priceChangePercentage24h: .zero, updatedAt: .now),
                value: transaction.transaction.valueBigInt,
                currency: Currency.usd.rawValue,
                formatter: formatter,
                textStyle: TextStyle(font: .body, color: Colors.black, fontWeight: .medium),
            ).fiat
        case let .perpetualPnl(pnl):
            return AmountDisplay.currency(value: pnl, currencyCode: Currency.usd.rawValue)
        }
    }

    private func swapAmount(assetId: AssetId?, value: String?, sign: AmountDisplaySign, textStyle: TextStyle?) -> TextValue? {
        guard
            let assetId,
            let value,
            let asset = transaction.assets.first(where: { $0.id == assetId }),
            let amount = try? BigInt.from(string: value)
        else {
            return .none
        }
        return AmountDisplay.numeric(
            data: AssetValuePrice(asset: asset, value: amount, price: nil),
            style: AmountDisplayStyle(sign: sign, formatter: formatter, currencyCode: currency, textStyle: textStyle),
        ).amount
    }

    private var swapMetadata: TransactionSwapMetadata? {
        transaction.transaction.metadata?.decode(TransactionSwapMetadata.self)
    }

    private var assetId: AssetId {
        transaction.transaction.assetId
    }

    public func getAddressName(address: String) -> AddressName? {
        if address == transaction.transaction.from {
            return transaction.fromAddress
        }

        if address == transaction.transaction.to {
            return transaction.toAddress
        }

        return .none
    }

    // MARK: - Private methods

    private func getDisplayName(address: String, chain: Chain) -> String {
        guard address.isNotEmpty else { return "" }
        if let name = getAddressName(address: address)?.name {
            return name
        }
        return AddressFormatter(address: address, chain: chain).value()
    }

    private func participantTitle(prefix: String, address: String, chain: Chain) -> String? {
        let value = getDisplayName(address: address, chain: chain)
        guard value.isNotEmpty else { return nil }
        return String(format: "%@ %@", prefix, value)
    }

    private func resourceTitle(prefix: String, resource: Gemstone.Resource) -> String {
        String(format: "%@ %@", prefix, ResourceViewModel(resource: resource.map()).title)
    }
}
