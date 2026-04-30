// Copyright (c). Gem Wallet. All rights reserved.

@testable import PrimitivesComponents
import Testing

struct HeaderBannerEventViewModelTests {
    @Test func isEnabled() {
        #expect(HeaderBannerEventViewModel(events: [.stake]).isButtonsEnabled == true)
        #expect(HeaderBannerEventViewModel(events: [.enableNotifications]).isButtonsEnabled == true)
        #expect(HeaderBannerEventViewModel(events: [.accountActivation]).isButtonsEnabled == true)
        #expect(HeaderBannerEventViewModel(events: [.accountActivation, .activateAsset]).isButtonsEnabled == false)
        #expect(HeaderBannerEventViewModel(events: [.stake, .activateAsset]).isButtonsEnabled == false)
        #expect(HeaderBannerEventViewModel(events: [.accountBlockedMultiSignature]).isButtonsEnabled == false)
    }
}
