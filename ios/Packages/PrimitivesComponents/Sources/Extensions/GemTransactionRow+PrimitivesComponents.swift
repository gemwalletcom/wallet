// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemTransactionRow
import enum Gemstone.Resource
import func Gemstone.transactionRows
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

extension GemTransactionRow: @retroactive Identifiable {}

public func transactionListSections(_ transactions: [TransactionListItem]) -> [ListSection<GemTransactionRow>] {
    DateSectionBuilder(items: transactionRows(items: transactions.map { $0.toGem() }), dateKeyPath: \.createdAt).build()
}

public extension GemTransactionRow {
    var transactionId: TransactionId {
        TransactionId(core: id)
    }

    var assetImage: AssetImage {
        let asset = AssetImage(icon: icon)
        return AssetImage(
            type: asset.type,
            imageURL: asset.imageURL,
            placeholder: asset.placeholder,
            chainPlaceholder: overlayImage,
        )
    }

    var overlayImage: Image? {
        switch badge {
        case .incoming: Images.Transaction.incoming
        case .outgoing: Images.Transaction.outgoing
        case .asset: AssetImage(icon: icon).chainPlaceholder
        }
    }

    var listItem: ListItemModel {
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

    var titleTextValue: TextValue {
        TextValue(
            text: title.title,
            style: TextStyle(font: Font.system(.body, weight: .medium), color: .primary),
        )
    }

    var titleTagType: TitleTagType {
        status.showsProgress ? .progressView() : .none
    }

    var titleTagTextValue: TextValue? {
        let model = TransactionStateViewModel(state: state.toPrimitives(), tone: status.tone)
        let title: String? = status.showsBadge ? model.title : .none
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

    var titleExtraTextValue: TextValue? {
        let prefix = subtitle.prefix ?? ""
        let title: String? = switch subtitle {
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

    var subtitleTextValue: TextValue? {
        value.textValue(textStyle: TextStyle(font: .body, color: valueTone.color, fontWeight: .medium))
    }

    var subtitleExtraTextValue: TextValue? {
        equivalentValue.textValue(textStyle: .footnote)
    }

    private func participantTitle(prefix: String, participant: String) -> String? {
        guard participant.isNotEmpty else { return nil }
        return String(format: "%@ %@", prefix, participant)
    }

    private func resourceTitle(prefix: String, resource: Gemstone.Resource) -> String {
        String(format: "%@ %@", prefix, resource.toPrimitives().title)
    }
}
