// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.SocialUrl
import GemstonePrimitives
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import WalletSessionService

@Observable
@MainActor
public final class SettingsViewModel {
    private let walletId: WalletId
    private let walletSessionService: any WalletSessionManageable
    private let observablePreferences: ObservablePreferences

    public init(
        walletId: WalletId,
        walletSessionService: any WalletSessionManageable,
        observablePreferences: ObservablePreferences,
    ) {
        self.walletId = walletId
        self.walletSessionService = walletSessionService
        self.observablePreferences = observablePreferences
    }

    var isDeveloperEnabled: Bool {
        observablePreferences.isDeveloperEnabled
    }

    var title: String {
        Localized.Settings.title
    }

    var walletsTitle: String {
        Localized.Wallets.title
    }

    var walletsValue: String {
        let count = (try? walletSessionService.walletsCount()) ?? .zero
        return "\(count)"
    }

    var walletsImage: AssetImage {
        AssetImage.image(Images.Settings.wallets)
    }

    var securityTitle: String {
        Localized.Settings.security
    }

    var securityImage: AssetImage {
        AssetImage.image(Images.Settings.security)
    }

    var notificationsTitle: String {
        Localized.Settings.Notifications.title
    }

    var notificationsImage: AssetImage {
        AssetImage.image(Images.Settings.notifications)
    }

    var preferencesTitle: String {
        Localized.Settings.Preferences.title
    }

    var preferencesImage: AssetImage {
        AssetImage.image(Images.Settings.preferences)
    }

    var walletConnectTitle: String {
        Localized.WalletConnect.title
    }

    var walletConnectImage: AssetImage {
        AssetImage.image(Images.Settings.walletConnect)
    }

    var rewardsTitle: String {
        Localized.Rewards.title
    }

    var rewardsImage: AssetImage {
        AssetImage.image(Images.Settings.gem)
    }

    var showsRewards: Bool {
        walletSessionService.hasMulticoinWallet()
    }

    var aboutUsTitle: String {
        Localized.Settings.aboutus
    }

    var aboutUsImage: AssetImage {
        AssetImage.image(Images.Settings.aboutUs)
    }

    var helpCenterTitle: String {
        Localized.Settings.helpCenter
    }

    var helpCenterImage: AssetImage {
        AssetImage.image(Images.Settings.helpCenter)
    }

    var helpCenterURL: URL {
        AppUrl.docs(.start)
    }

    var supportTitle: String {
        Localized.Settings.support
    }

    var supportImage: AssetImage {
        AssetImage.image(Images.Settings.support)
    }

    var supportURL: URL {
        AppUrl.page(.support)
    }

    var developerModeTitle: String {
        Localized.Settings.developer
    }

    var developerModeImage: AssetImage {
        AssetImage.image(Images.Settings.developer)
    }
}
