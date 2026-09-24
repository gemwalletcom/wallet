// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemSecurityReminderItem
import GemstonePrimitives
import Localization
import Style

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
        ListItemModel(title: item.title, titleLineLimit: 2, titleExtra: item.subtitle, titleStyleExtra: .bodySecondary, imageStyle: item.image)
    }

    var items: [GemSecurityReminderItem] = GemConstants.securityReminderItems
    var docsUrl: URL {
        AppUrl.docs(.whatIsSecretPhrase)
    }
}
