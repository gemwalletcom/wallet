// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemNotificationRow
import PrimitivesComponents
import Style

extension GemNotificationRow: @retroactive Identifiable {}

extension GemNotificationRow {
    var listItem: ListItemModel {
        ListItemModel(
            title: title,
            titleTag: tag?.text,
            titleTagStyle: TextStyle(font: .footnote.weight(.medium), color: .blue, background: Colors.blue.opacity(.light)),
            titleExtra: subtitle,
            subtitle: value,
            subtitleStyle: .callout,
            subtitleExtra: subvalue,
            imageStyle: icon.flatMap {
                ListItemImageStyle(
                    assetImage: $0.assetImage,
                    imageSize: .image.asset,
                    alignment: .top,
                    cornerRadiusType: .rounded,
                )
            },
        )
    }
}
