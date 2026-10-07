// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemSecurityReminderItem
import enum Gemstone.GemSecurityReminderKind
import func Gemstone.securityReminderItems
import GemstonePrimitives
import Style

@Observable
public final class SecurityReminderViewModel {
    let title: String
    let kind: GemSecurityReminderKind
    let onNext: () -> Void

    public init(
        title: String,
        kind: GemSecurityReminderKind,
        onNext: @escaping () -> Void,
    ) {
        self.title = title
        self.kind = kind
        self.onNext = onNext
    }

    var message: String {
        kind.reminderIntro
    }

    func listItem(for item: GemSecurityReminderItem) -> ListItemModel {
        ListItemModel(title: item.title, titleLineLimit: 2, titleExtra: item.subtitle, titleStyleExtra: .bodySecondary, imageStyle: item.image)
    }

    var items: [GemSecurityReminderItem] {
        securityReminderItems(kind: kind)
    }

    var docsUrl: URL {
        kind.reminderDocsUrl
    }
}
