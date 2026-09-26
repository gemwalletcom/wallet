// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemToast
import Localization
@testable import PrimitivesComponents
import Style
import Testing

struct ToastMessageTests {
    @Test
    func copy() {
        #expect(ToastMessage.copy("Copied").image == SystemImage.copy)
    }

    @Test
    func coreToast() {
        let unpinned = ToastMessage(toast: GemToast(text: .pinned(name: "BTC", pinned: false), icon: .unpin))

        #expect(unpinned.title == Localized.Common.unpinnedAsset("BTC"))
        #expect(unpinned.image == SystemImage.unpin)
        #expect(ToastMessage(toast: GemToast(text: .priceAlertsToggled(name: "ETH", enabled: true), icon: .priceAlert)).image == SystemImage.bellFill)
    }

    @Test
    func addedToWallet() {
        #expect(ToastMessage.addedToWallet().image == SystemImage.plusCircle)
    }

    @Test
    func showAsset() {
        #expect(ToastMessage.showAsset(visible: true).image == SystemImage.plusCircle)
        #expect(ToastMessage.showAsset(visible: false).image == SystemImage.minusCircle)
    }

    @Test
    func priceAlert() {
        #expect(ToastMessage.priceAlert(message: "Alert set").image == SystemImage.bellFill)
    }

    @Test
    func success() {
        #expect(ToastMessage.success("Saved").image == SystemImage.checkmark)
    }
}
