// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemErrorText
import struct Gemstone.GemPushState
import GemstonePrimitivesTestKit
import PrimitivesComponents
@testable import Settings
import Testing

@MainActor
struct NotificationsViewModelTests {
    @Test
    func theToggleFollowsTheStateCoreAnswersWith() async {
        let service = GemNotificationsServiceMock(state: GemPushState(isEnabled: false, result: .permissionDenied))
        let model = NotificationsViewModel(service: service)

        await model.enable(isEnabled: true)

        #expect(service.requested == [true])
        #expect(model.isEnabled == false, "a declined permission leaves the toggle where the preference is")
        #expect(model.isPresentingAlertMessage == nil, "declining is not an error to alert about")
    }

    @Test
    func aRegistrationCoreCouldNotFinishAlertsItsOwnText() async {
        let service = GemNotificationsServiceMock(state: GemPushState(isEnabled: true, result: .notRegistered(error: GemErrorText.networkOffline)))
        let model = NotificationsViewModel(service: service)

        await model.enable(isEnabled: true)

        #expect(model.isEnabled == true, "the preference Core stored is what the toggle shows")
        #expect(model.isPresentingAlertMessage?.message == GemErrorText.networkOffline.text)
    }
}
