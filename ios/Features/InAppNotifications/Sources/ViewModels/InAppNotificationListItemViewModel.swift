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
    private let item: CoreListItem
    private let row: GemNotificationRow

    public let id: String
    public let url: URL?

    public init(notification: InAppNotification) {
        id = notification.id
        item = notification.item
        row = notificationRow(notification: notification.map())
        url = notification.item.url?.asURL
    }

    var listItemModel: ListItemModel {
        ListItemModel(
            title: item.title,
            titleTag: row.isUnread ? Localized.Assets.Tags.new : nil,
            titleTagStyle: TextStyle(font: .footnote.weight(.medium), color: .blue, background: Colors.blue.opacity(.light)),
            titleExtra: item.subtitle,
            subtitle: item.value,
            subtitleStyle: TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold),
            subtitleExtra: item.subvalue,
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
