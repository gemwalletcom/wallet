// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAddressService
import enum Gemstone.Resource
import Components
import Formatters
import Foundation
import class Gemstone.GemTransactionRow
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct TransactionViewModel: Sendable {
    public let transaction: TransactionExtended
    private let row: GemTransactionRow
    private let currency: String
    private let formatter: ValueFormatter = .short

    public init(
        transaction: TransactionExtended,
        currency: String,
    ) {
        row = GemTransactionRow(transaction: transaction.json())
        self.transaction = transaction
        self.currency = currency
    }

    public var assetImage: AssetImage {
        let asset = AssetIdViewModel(assetId: assetId).assetImage
        if let nftImageUrl = row.nftImageUrl() {
            return AssetImage(
                type: .text(""),
                imageURL: URL(string: nftImageUrl),
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
        case let .toAddress(address, name): participantTitle(prefix: Localized.Transfer.to, address: address, name: name)
        case let .fromAddress(address, name): participantTitle(prefix: Localized.Transfer.from, address: address, name: name)
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
        row.value().textValue(currency: currency, formatter: formatter)
    }

    public var subtitleExtraTextValue: TextValue? {
        row.equivalentValue().textValue(currency: currency, formatter: formatter, textStyle: .footnote)
    }

    private var assetId: AssetId {
        transaction.transaction.assetId
    }

    private func participantTitle(prefix: String, address: String, name: String?) -> String? {
        guard address.isNotEmpty else { return nil }
        let value = name ?? GemAddressService.shared.format(address: address, chain: assetId.chain)
        return String(format: "%@ %@", prefix, value)
    }

    private func resourceTitle(prefix: String, resource: Gemstone.Resource) -> String {
        String(format: "%@ %@", prefix, ResourceViewModel(resource: resource.map()).title)
    }
}
