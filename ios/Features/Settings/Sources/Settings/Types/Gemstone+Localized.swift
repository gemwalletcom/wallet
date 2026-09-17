// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAddNodeFailure
import enum Gemstone.GemServiceEndpointType
import enum Gemstone.GemAboutRow
import enum Gemstone.GemPreferencesRow
import enum Gemstone.GemSettingsRow
import enum Gemstone.GemChainSettingsSection
import enum Gemstone.GemNodeCheckRow
import enum Gemstone.GemNodeSubtitle
import enum GemstoneServices.KeystoreAuthentication
import Localization
import Primitives

public extension Appearance {
    var title: String {
        switch self {
        case .system: Localized.Settings.appearanceSystem
        case .light: Localized.Settings.appearanceLight
        case .dark: Localized.Settings.appearanceDark
        }
    }
}

extension GemServiceEndpointType {
    var name: String {
        switch self {
        case .api: "API"
        case .gemNode: Localized.Nodes.gemWalletNode
        }
    }
}

extension GemSettingsRow {
    var title: String {
        switch self {
        case .wallets: Localized.Wallets.title
        case .security: Localized.Settings.security
        case .notifications: Localized.Settings.Notifications.title
        case .preferences: Localized.Settings.Preferences.title
        case .walletConnect: Localized.WalletConnect.title
        case .support: Localized.Settings.support
        case .rewards: Localized.Rewards.title
        case .aboutUs: Localized.Settings.aboutus
        case .developer: Localized.Settings.developer
        }
    }
}

extension GemPreferencesRow {
    var title: String {
        switch self {
        case .currency: Localized.Settings.currency
        case .language: Localized.Settings.language
        case .appearance: Localized.Settings.appearanceTitle
        case .networks: Localized.Settings.Networks.title
        case .contacts: Localized.Contacts.title
        case .perpetuals: Localized.Perpetuals.title
        case .perpetualLeverage: Localized.Settings.Preferences.Perpetual.defaultLeverage
        case .perpetualTakeProfit: Localized.Settings.Preferences.Perpetual.defaultTakeProfit
        case .perpetualStopLoss: Localized.Settings.Preferences.Perpetual.defaultStopLoss
        }
    }
}

extension GemAboutRow {
    var title: String {
        switch self {
        case .termsOfService: Localized.Settings.termsOfServices
        case .privacyPolicy: Localized.Settings.privacyPolicy
        case .website: Localized.Settings.website
        case .community: Localized.Settings.community
        case .version: Localized.Settings.version
        }
    }
}

extension GemChainSettingsSection {
    var title: String {
        switch self {
        case .nodes: Localized.Settings.Networks.source
        case .explorer: Localized.Settings.Networks.explorer
        }
    }
}

extension GemNodeCheckRow {
    var title: String {
        switch self {
        case .chainId: Localized.Nodes.ImportNode.chainId
        case .inSync: Localized.Nodes.ImportNode.inSync
        case .latestBlock: Localized.Nodes.ImportNode.latestBlock
        case .latency: Localized.Nodes.ImportNode.latency
        }
    }

    var text: String {
        switch self {
        case let .chainId(value), let .latestBlock(value): value
        case let .inSync(state): state.symbol
        case let .latency(milliseconds): Localized.Common.latencyInMs(Int(milliseconds))
        }
    }
}

extension GemNodeSubtitle {
    var title: String {
        switch self {
        case .latestBlock: Localized.Nodes.ImportNode.latestBlock
        }
    }
}

extension KeystoreAuthentication {
    var enableTitle: String {
        switch self {
        case .biometrics:
            if let name = KeystoreAuthentication.availableBiometryName {
                Localized.Settings.enableValue(name)
            } else {
                Localized.Settings.enablePasscode
            }
        case .passcode, .none: Localized.Settings.enablePasscode
        }
    }
}

extension GemAddNodeFailure {
    var error: AnyError {
        switch self {
        case .invalidUrl: AnyError(Localized.Errors.invalidUrl)
        case .invalidNetworkId: AnyError(Localized.Errors.invalidNetworkId)
        case .unavailable: AnyError(Localized.Errors.errorOccurred)
        }
    }
}
