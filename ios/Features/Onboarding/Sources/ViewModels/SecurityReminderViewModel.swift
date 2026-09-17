// Copyright (c). Gem Wallet. All rights reserved.

import Style
import Components
import Foundation
import enum Gemstone.GemSecurityReminderItem
import func Gemstone.securityReminderItems
import GemstonePrimitives
import Localization

@Observable
final class SecurityReminderViewModel {
    let title: String
    let onNext: () -> Void

    init(
        title: String,
        onNext: @escaping () -> Void,
    ) {
        self.title = title
        self.onNext = onNext
    }

    var message: String = Localized.Onboarding.Security.CreateWallet.Intro.title
    func listItem(for item: GemSecurityReminderItem) -> ListItemModel {
        ListItemModel(title: item.title, titleStyle: .headline, titleLineLimit: 2, titleExtra: item.subtitle, titleStyleExtra: .bodySecondary, imageStyle: item.image)
    }

    var items: [GemSecurityReminderItem] = securityReminderItems()
    var docsUrl: URL {
        AppUrl.docs(.whatIsSecretPhrase)
    }
}
