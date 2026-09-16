// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Testing
@testable import Gem

@MainActor
struct AppUpdateAlertTests {
    @Test
    func aRequiredUpgradeOffersOnlyTheUpdate() {
        let release = Release(version: "2.0.0", store: .appStore, upgradeRequired: true)

        let actions = RootSceneViewModel.updateAlertActions(for: release, skip: .skip, update: .update)

        #expect(actions.map(\.title) == [AlertAction.update.title])
    }

    @Test
    func anOptionalUpgradeAlsoOffersSkip() {
        let release = Release(version: "2.0.0", store: .appStore, upgradeRequired: false)

        let actions = RootSceneViewModel.updateAlertActions(for: release, skip: .skip, update: .update)

        #expect(actions.map(\.title) == [AlertAction.skip.title, AlertAction.update.title])
    }
}

private extension AlertAction {
    static let skip = AlertAction(title: "skip", role: .cancel, action: {})
    static let update = AlertAction(title: "update", isDefaultAction: true, action: {})
}
