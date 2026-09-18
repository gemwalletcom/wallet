// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAddNodeFailure
import enum Gemstone.GemServiceEndpointType
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
