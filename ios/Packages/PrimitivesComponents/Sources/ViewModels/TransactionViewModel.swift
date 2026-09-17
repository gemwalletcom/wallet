// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.Resource
import Components
import Formatters
import Foundation
import struct Gemstone.GemTransactionRow
import func Gemstone.transactionRow
import func Gemstone.transactionRows
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct TransactionViewModel: Sendable, Identifiable, Equatable {
    public let transaction: TransactionExtended
    private let row: GemTransactionRow

    public init(transaction: TransactionExtended) {
        self.init(transaction: transaction, row: transactionRow(transaction: transaction.toGem()))
    }

    public init(transaction: TransactionExtended, row: GemTransactionRow) {
        self.transaction = transaction
        self.row = row
    }

    public var id: String {
        transaction.id
    }

    public static func sections(_ transactions: [TransactionExtended]) -> [ListSection<TransactionViewModel>] {
        let models = zip(transactions, transactionRows(transactions: transactions.map { $0.toGem() }))
            .map { TransactionViewModel(transaction: $0, row: $1) }
        return DateSectionBuilder(items: models, dateKeyPath: \.transaction.transaction.createdAt).build()
    }

    public var assetImage: AssetImage {
        let asset = AssetIdViewModel(assetId: assetId).assetImage
        if let nftImageUrl = row.nftImageUrl {
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

    public func listItem(currency: Currency) -> ListItemModel {
        let title = titleTextValue
        let titleExtra = titleExtraTextValue
        let titleTag = titleTagTextValue
        let subtitle = subtitleTextValue(currency: currency)
        let subtitleExtra = subtitleExtraTextValue(currency: currency)
        return ListItemModel(
            title: title.text,
            titleStyle: title.style,
            titleLineLimit: title.lineLimit,
            titleTag: titleTag?.text,
            titleTagStyle: titleTag?.style ?? ListItemModel.StyleDefaults.titleTagStyle,
            titleTagLineLimit: titleTag?.lineLimit,
            titleTagType: titleTagType,
            titleExtra: titleExtra?.text,
            titleStyleExtra: titleExtra?.style ?? ListItemModel.StyleDefaults.titleExtraStyle,
            titleExtraLineLimit: titleExtra?.lineLimit,
            subtitle: subtitle?.text,
            subtitleStyle: subtitle?.style ?? ListItemModel.StyleDefaults.subtitleStyle,
            subtitleLineLimit: subtitle?.lineLimit,
            subtitleExtra: subtitleExtra?.text,
            subtitleStyleExtra: subtitleExtra?.style ?? ListItemModel.StyleDefaults.subtitleExtraStyle,
            subtitleExtraLineLimit: subtitleExtra?.lineLimit,
            imageStyle: .asset(assetImage: assetImage),
        )
    }

    public var titleTextValue: TextValue {
        TextValue(
            text: row.title.title,
            style: TextStyle(font: Font.system(.body, weight: .medium), color: .primary),
        )
    }

    public var titleTagType: TitleTagType {
        row.status.showsProgress ? .progressView() : .none
    }

    public var titleTagTextValue: TextValue? {
        let model = TransactionStateViewModel(state: transaction.transaction.state, tone: row.status.tone)
        let title: String? = row.status.showsBadge ? model.title : .none
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
        let prefix = row.subtitle.prefix ?? ""
        let title: String? = switch row.subtitle {
        case let .toAddress(participant), let .fromAddress(participant): participantTitle(prefix: prefix, participant: participant)
        case let .toResource(resource), let .fromResource(resource): resourceTitle(prefix: prefix, resource: resource)
        case let .price(value):
            String(format: "%@: %@", prefix, AmountDisplay.currency(value: value, currencyCode: Currency.usd.rawValue, showSign: false).text)
        case .none: .none
        }

        return title.map {
            TextValue(
                text: $0,
                style: .footnote,
            )
        }
    }

    public func subtitleTextValue(currency: Currency) -> TextValue? {
        row.value.textValue(currency: currency, formatter: .short)
    }

    public func subtitleExtraTextValue(currency: Currency) -> TextValue? {
        row.equivalentValue.textValue(currency: currency, formatter: .short, textStyle: .footnote)
    }

    private var assetId: AssetId {
        transaction.transaction.assetId
    }

    private func participantTitle(prefix: String, participant: String) -> String? {
        guard participant.isNotEmpty else { return nil }
        return String(format: "%@ %@", prefix, participant)
    }

    private func resourceTitle(prefix: String, resource: Gemstone.Resource) -> String {
        String(format: "%@ %@", prefix, resource.toPrimitives().title)
    }
}
