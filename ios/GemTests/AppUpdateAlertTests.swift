// Copyright (c). Gem Wallet. All rights reserved.

import Components
@testable import Gem
import Primitives
import Testing

@MainActor
struct AppUpdateAlertTests {
    @Test
    func aRequiredUpgradeOffersOnlyTheUpdate() {
        let release = Release(version: "2.0.0", store: .appStore, upgradeRequired: true)

        let actions = RootSceneViewModel.updateAlertActions(
            for: release,
            skip: AlertAction(title: "skip", role: .cancel, action: {}),
            update: AlertAction(title: "update", isDefaultAction: true, action: {}),
        )

        #expect(actions.map(\.title) == ["update"])
    }

    @Test
    func anOptionalUpgradeAlsoOffersSkip() {
        let release = Release(version: "2.0.0", store: .appStore, upgradeRequired: false)

        let actions = RootSceneViewModel.updateAlertActions(
            for: release,
            skip: AlertAction(title: "skip", role: .cancel, action: {}),
            update: AlertAction(title: "update", isDefaultAction: true, action: {}),
        )

        #expect(actions.map(\.title) == ["skip", "update"])
    }
}
