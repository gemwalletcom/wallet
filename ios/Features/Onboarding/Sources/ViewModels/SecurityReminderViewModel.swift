// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
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
    var items: [SecurityReminderItem] = SecurityReminderItem.createWallet
    var docsUrl: URL {
        AppUrl.docs(.whatIsSecretPhrase)
    }
}
