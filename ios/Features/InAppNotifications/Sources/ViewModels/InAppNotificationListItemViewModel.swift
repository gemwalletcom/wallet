// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemNotificationDestination
import enum Gemstone.GemNotificationIcon
import struct Gemstone.GemNotificationRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style

public struct InAppNotificationListItemViewModel: Identifiable, Sendable {
    private let row: GemNotificationRow

    public let id: String
    let createdAt: Date

    public init(notification: InAppNotification, row: GemNotificationRow) {
        id = notification.id
        createdAt = notification.createdAt
        self.row = row
    }

    public var destination: GemNotificationDestination? {
        row.destination
    }

    var listItemModel: ListItemModel {
        ListItemModel(
            title: row.title,
            titleTag: row.isUnread ? Localized.Assets.Tags.new : nil,
            titleTagStyle: TextStyle(font: .footnote.weight(.medium), color: .blue, background: Colors.blue.opacity(.light)),
            titleExtra: row.subtitle,
            subtitle: row.value,
            subtitleStyle: .callout,
            subtitleExtra: row.subvalue,
            imageStyle: imageStyle,
        )
    }

    private var imageStyle: ListItemImageStyle? {
        guard let icon = row.icon else { return nil }
        return ListItemImageStyle(
            assetImage: assetImage(for: icon),
            imageSize: .image.asset,
            alignment: .top,
            cornerRadiusType: .rounded,
        )
    }

    private func assetImage(for icon: GemNotificationIcon) -> AssetImage {
        switch icon {
        case let .emoji(glyph): AssetImage(type: .emoji(glyph))
        case let .asset(_, icon): AssetImage(icon: icon)
        case let .image(url): AssetImage(imageURL: url.asURL)
        }
    }
}
