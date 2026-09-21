// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemCurrencySectionKind
import enum Gemstone.GemNodeCheckRow
import enum Gemstone.GemNodeSubtitle
import GemstonePrimitives
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
        case let .chainId(value): value
        case let .latestBlock(value): value.text()
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

    var text: String {
        switch self {
        case let .latestBlock(value): text(latestBlockLabel: title, latestBlockValue: value?.text())
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

extension GemCurrencySectionKind {
    var title: String {
        switch self {
        case .recommended: Localized.Common.recommended
        case .all: Localized.Common.all
        }
    }
}
