// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemErrorText
import struct Gemstone.GemPushState
import GemstonePrimitivesTestKit
import PrimitivesComponents
@testable import Settings
import Testing

@MainActor
struct NotificationsSceneViewModelTests {
    @Test
    func theToggleFollowsTheStateCoreAnswersWith() async {
        let service = GemNotificationsServiceMock(state: GemPushState(isEnabled: false, result: .permissionDenied))
        let model = NotificationsSceneViewModel(service: service)

        await model.enable(isEnabled: true)

        #expect(service.requested == [true])
        #expect(model.isEnabled == false, "a declined permission leaves the toggle where the preference is")
        #expect(model.isPresentingAlertMessage == nil, "declining is not an error to alert about")
    }

    @Test
    func aRegistrationCoreCouldNotFinishAlertsItsOwnText() async {
        let service = GemNotificationsServiceMock(state: GemPushState(isEnabled: true, result: .notRegistered(error: GemErrorText.networkOffline)))
        let model = NotificationsSceneViewModel(service: service)

        await model.enable(isEnabled: true)

        #expect(model.isEnabled == true, "the preference Core stored is what the toggle shows")
        #expect(model.isPresentingAlertMessage?.message == GemErrorText.networkOffline.text)
    }

    @Test
    func theToggleStartsFromTheStoredChoice() {
        let model = NotificationsSceneViewModel(service: GemNotificationsServiceMock(state: GemPushState(isEnabled: true, result: .stored)))

        #expect(model.isEnabled == true, "no off-then-on flicker while the permission is read")
    }

    @Test
    func returningToTheAppShowsWhatCoreReadsFromTheSystemPermission() async {
        let service = GemNotificationsServiceMock(state: GemPushState(isEnabled: true, result: .stored))
        let model = NotificationsSceneViewModel(service: service)

        await model.refresh()
        #expect(model.isEnabled == true)

        service.state = GemPushState(isEnabled: false, result: .stored)
        await model.refresh()

        #expect(model.isEnabled == false, "a permission revoked in system settings turns the toggle off")
        #expect(service.requested.isEmpty, "re-reading never stores a choice the user did not make")
    }
}
