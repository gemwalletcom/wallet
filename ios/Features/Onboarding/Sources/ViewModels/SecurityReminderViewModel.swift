// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.phraseSecurityReminder
import func Gemstone.privateKeySecurityReminder
import struct Gemstone.GemSecurityReminder
import GemstonePrimitives
import Localization
import Primitives

@Observable
final class SecurityReminderViewModel {
    let title: String
    let message: String
    let items: [SecurityReminderItem]
    let docsUrl: URL
    let onNext: () -> Void

    convenience init(
        title: String,
        onNext: @escaping () -> Void,
    ) {
        self.init(
            title: title,
            message: Localized.Onboarding.Security.CreateWallet.Intro.title,
            reminder: phraseSecurityReminder(),
            onNext: onNext,
        )
    }

    convenience init(
        chain: Chain,
        onNext: @escaping () -> Void,
    ) {
        self.init(
            title: Localized.Common.privateKey,
            message: Localized.PrivateKey.revealIntro(chain.networkName),
            reminder: privateKeySecurityReminder(chain: chain.rawValue),
            privateKeyChain: chain.networkName,
            onNext: onNext,
        )
    }

    private init(
        title: String,
        message: String,
        reminder: GemSecurityReminder,
        privateKeyChain: String? = nil,
        onNext: @escaping () -> Void,
    ) {
        self.title = title
        self.message = message
        items = SecurityReminderItem.items(reminder, privateKeyChain: privateKeyChain)
        docsUrl = AppUrl.docs(reminder.docsUrl)
        self.onNext = onNext
    }
}
