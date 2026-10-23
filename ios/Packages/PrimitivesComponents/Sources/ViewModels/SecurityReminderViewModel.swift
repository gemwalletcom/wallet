// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemSecurityReminderItem
import enum Gemstone.GemSecurityReminderKind
import func Gemstone.securityReminderItems
import GemstonePrimitives
import Localization
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
        switch kind {
        case .phrase: Localized.Onboarding.Security.CreateWallet.Intro.title
        case .privateKey: Localized.PrivateKey.Reminder.intro
        case .accountKey: Localized.PrivateKey.ExportReminder.intro
        }
    }

    func listItem(for item: GemSecurityReminderItem) -> ListItemModel {
        ListItemModel(title: item.title, titleLineLimit: 2, titleExtra: item.subtitle, titleStyleExtra: .bodySecondary, imageStyle: item.image)
    }

    var items: [GemSecurityReminderItem] {
        securityReminderItems(kind: kind)
    }

    var docsUrl: URL {
        switch kind {
        case .phrase: AppUrl.docs(.whatIsSecretPhrase)
        case .privateKey, .accountKey: AppUrl.docs(.whatIsPrivateKey)
        }
    }
}
