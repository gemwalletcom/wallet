// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Localization

@Observable
final class SecurityReminderViewModelDefault: SecurityReminderViewModel {
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
    var checkmarkTitle: String = Localized.Onboarding.Security.CreateWallet.Confirm.title
    var buttonTitle: String = Localized.Common.continue
    var items: [SecurityReminderItem] = SecurityReminderItem.createWallet
    var docsUrl: URL {
        AppUrl.docs(.whatIsSecretPhrase)
    }
}
