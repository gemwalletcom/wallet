// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemTransactionRow
import enum Gemstone.Resource
import func Gemstone.transactionRow
import func Gemstone.transactionRows
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct TransactionViewModel: Sendable, Identifiable, Equatable {
    private let row: GemTransactionRow

    public init(transaction: TransactionExtended) {
        self.init(row: transactionRow(transaction: transaction.toGem()))
    }

    public init(row: GemTransactionRow) {
        self.row = row
    }

    public var id: String {
        row.id
    }

    public var transactionId: TransactionId {
        TransactionId(core: row.id)
    }

    public var createdAt: Date {
        row.createdAt
    }

    public static func sections(_ transactions: [TransactionExtended]) -> [ListSection<TransactionViewModel>] {
        let models = transactionRows(transactions: transactions.map { $0.toGem() }).map(TransactionViewModel.init(row:))
        return DateSectionBuilder(items: models, dateKeyPath: \.createdAt).build()
    }

    public var assetImage: AssetImage {
        let asset = AssetImage(icon: row.icon)
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
        switch row.badge {
        case .incoming: Images.Transaction.incoming
        case .outgoing: Images.Transaction.outgoing
        case .asset: AssetImage(icon: row.icon).chainPlaceholder
        }
    }

    public var listItem: ListItemModel {
        let title = titleTextValue
        let titleExtra = titleExtraTextValue
        let titleTag = titleTagTextValue
        let subtitle = subtitleTextValue
        let subtitleExtra = subtitleExtraTextValue
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
        let model = TransactionStateViewModel(state: row.state.toPrimitives(), tone: row.status.tone)
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
        case let .price(price):
            String(format: "%@: %@", prefix, price.text())
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
        row.value.textValue(textStyle: TextStyle(font: .body, color: row.valueTone.color, fontWeight: .medium))
    }

    public var subtitleExtraTextValue: TextValue? {
        row.equivalentValue.textValue(textStyle: .footnote)
    }

    private var assetId: AssetId {
        row.asset.toPrimitives().id
    }

    private func participantTitle(prefix: String, participant: String) -> String? {
        guard participant.isNotEmpty else { return nil }
        return String(format: "%@ %@", prefix, participant)
    }

    private func resourceTitle(prefix: String, resource: Gemstone.Resource) -> String {
        String(format: "%@ %@", prefix, resource.toPrimitives().title)
    }
}
