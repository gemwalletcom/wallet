// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemPrivateKeyScope
import enum Gemstone.GemSecurityReminderItem
import struct Gemstone.GemSecurityReminder
import Localization
import Style

struct SecurityReminderItem: Identifiable {
    var id: String {
        [title, subtitle].compactMap { $0 }.joined()
    }

    let title: String
    let subtitle: String?
    let image: ListItemImageStyle?
}

extension SecurityReminderItem {
    static func items(_ reminder: GemSecurityReminder, privateKeyChain: String? = nil) -> [Self] {
        reminder.items.map { SecurityReminderItem($0, privateKeyChain: privateKeyChain) }
    }

    private init(_ item: GemSecurityReminderItem, privateKeyChain: String?) {
        switch item {
        case .keepSafe:
            self.init(
                title: Localized.Onboarding.Security.CreateWallet.KeepSafe.title,
                subtitle: privateKeyChain == nil ? Localized.Onboarding.Security.CreateWallet.KeepSafe.subtitle : nil,
                image: .security(Emoji.WalletAvatar.lock.rawValue),
            )
        case .doNotShare:
            self.init(
                title: Localized.Onboarding.Security.CreateWallet.DoNotShare.title,
                subtitle: privateKeyChain.map { Localized.PrivateKey.DoNotShare.description($0) } ?? Localized.Onboarding.Security.CreateWallet.DoNotShare.subtitle,
                image: .security(Emoji.WalletAvatar.warning.rawValue),
            )
        case .noRecovery:
            self.init(
                title: Localized.Onboarding.Security.CreateWallet.NoRecovery.title,
                subtitle: Localized.Onboarding.Security.CreateWallet.NoRecovery.subtitle,
                image: .security(Emoji.WalletAvatar.gem.rawValue),
            )
        case .keyScope(.oneChain):
            self.init(
                title: Localized.PrivateKey.OneChain.title,
                subtitle: privateKeyChain.map { Localized.PrivateKey.OneChain.description($0) },
                image: .security(Emoji.WalletAvatar.gem.rawValue),
            )
        case .keyScope(.evmChains):
            self.init(sharedChains: Localized.PrivateKey.Scope.evm)
        case .keyScope(.cosmosChains):
            self.init(sharedChains: Localized.PrivateKey.Scope.cosmos)
        }
    }

    private init(sharedChains: String) {
        self.init(
            title: Localized.PrivateKey.SharedChains.title,
            subtitle: Localized.PrivateKey.SharedChains.description(sharedChains),
            image: .security(Emoji.WalletAvatar.gem.rawValue),
        )
    }
}
