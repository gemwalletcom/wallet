// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemAcceptTermsItem
import enum Gemstone.GemSecurityReminderItem
import enum Gemstone.GemWalletImportKind
import enum Gemstone.GemWalletSecret
import Localization

extension GemWalletImportKind {
    var title: String {
        switch self {
        case .phrase: Localized.Common.phrase
        case .privateKey: Localized.Common.privateKey
        case .address: Localized.Common.address
        }
    }

    var description: String {
        switch self {
        case .phrase: Localized.Common.secretPhrase
        case .privateKey: Localized.Common.privateKey
        case .address: Localized.Wallet.Import.addressField
        }
    }
}

extension GemAcceptTermsItem {
    var message: String {
        switch self {
        case .selfCustody: Localized.Onboarding.AcceptTerms.Item1.message
        case .recovery: Localized.Onboarding.AcceptTerms.Item2.message
        case .responsibility: Localized.Onboarding.AcceptTerms.Item3.message
        }
    }
}

extension GemSecurityReminderItem {
    var title: String {
        switch self {
        case .keepSafe: Localized.Onboarding.Security.CreateWallet.KeepSafe.title
        case .doNotShare: Localized.Onboarding.Security.CreateWallet.DoNotShare.title
        case .noRecovery: Localized.Onboarding.Security.CreateWallet.NoRecovery.title
        }
    }

    var subtitle: String {
        switch self {
        case .keepSafe: Localized.Onboarding.Security.CreateWallet.KeepSafe.subtitle
        case .doNotShare: Localized.Onboarding.Security.CreateWallet.DoNotShare.subtitle
        case .noRecovery: Localized.Onboarding.Security.CreateWallet.NoRecovery.subtitle
        }
    }
}

extension GemWalletSecret {
    var title: String {
        switch self {
        case .words: Localized.Common.secretPhrase
        case .privateKey: Localized.Common.privateKey
        }
    }
}
