// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemNotificationIcon
import struct Gemstone.GemNotificationRow
import func Gemstone.notificationRow
import Localization
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style

public struct InAppNotificationListItemViewModel: Identifiable, Sendable {
    private let row: GemNotificationRow

    public let id: String

    public init(notification: InAppNotification) {
        id = notification.id
        row = notificationRow(notification: notification.toGem())
    }

    public var url: URL? {
        row.url?.asURL
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
        case let .asset(assetId): AssetIdViewModel(assetId: Primitives.AssetId(core: assetId)).assetImage
        case let .image(url): AssetImage(imageURL: url.asURL)
        }
    }
}
